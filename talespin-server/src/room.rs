use anyhow::{anyhow, Context, Result};
use axum::{extract::ws::Message as WsMessage, extract::ws::WebSocket};
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
};
use tokio::sync::{broadcast, mpsc, RwLock, RwLockWriteGuard};

const MODERATOR_ABSENCE_PROMOTION_DELAY_S: u64 = 5 * 60;

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
#[serde(tag = "mode", rename_all = "snake_case")]
pub enum WinCondition {
    Points { target_points: u16 },
    Cycles { target_cycles: u16 },
    CardsFinish,
}

#[derive(Debug, Serialize, Clone)]
pub enum ServerMsg {
    RoomState {
        room_id: String,
        players: HashMap<String, PlayerInfo>,
        creator: Option<String>,
        moderators: Vec<String>,
        stage: RoomStage,
        active_player: Option<String>,
        player_order: Vec<String>,
        round: u16,
        cards_remaining: u32,
        deck_refill_count: u32,
        win_condition: WinCondition,
    },
    StartRound {
        hand: Vec<String>,
    },
    PlayersChoose {
        description: String,
        hand: Vec<String>,
    },
    BeginVoting {
        center_cards: Vec<String>,
        description: String,
        disabled_card: Option<String>,
    },
    Results {
        player_to_vote: HashMap<String, String>,
        player_to_current_card: HashMap<String, String>,
        active_card: String,
        point_change: HashMap<String, u16>,
    },
    ErrorMsg(String),
    LeftRoom {
        reason: String,
    },
    Kicked {
        reason: String,
    },
    InvalidRoomId {},
    EndGame {},
}

impl From<ServerMsg> for WsMessage {
    fn from(msg: ServerMsg) -> Self {
        // this should never fail
        let json = serde_json::to_string(&msg).expect("Failed to serialize json");
        WsMessage::Text(json)
    }
}

#[derive(Debug, Deserialize)]
pub enum ClientMsg {
    Ready {},
    StartGame {},
    LeaveRoom {},
    KickPlayer { player: String },
    SetModerator { player: String, enabled: bool },
    JoinRoom { room_id: String, name: String },
    CreateRoom { name: String },
    ActivePlayerChooseCard { card: String, description: String },
    PlayerChooseCard { card: String },
    Vote { card: String },
    Ping {},
}

#[derive(Debug, Serialize, Clone, Copy)]
pub enum RoomStage {
    // waiting for players to join with room code
    Joining,
    // active player chooses card from their hand and writes description
    ActiveChooses,
    // players choose cards to match the description
    PlayersChoose,
    // players vote on what they think the active card is
    Voting,
    // results are computed; circle back to ActiveChooses while deck is not empty
    Results,
    // game is over
    End,
}

#[derive(Debug, Serialize, Clone)]
pub struct PlayerInfo {
    // player is connected to server
    connected: bool,
    // points in the game
    points: u16,
    // ready is stage-specific
    ready: bool, // this is round dependent
}

#[derive(Debug)]
struct RoomState {
    room_id: String,
    // lobby creator / host
    creator: Option<String>,
    // moderation privileges; creator is auto-included while present
    moderators: HashSet<String>,
    // when no moderators are connected, this starts the host-migration timer
    no_connected_moderator_since_s: Option<u64>,
    // players removed from this room explicitly (leave/kick)
    removed_players: HashSet<String>,
    // store general stats about each player
    players: HashMap<String, PlayerInfo>,
    // store 6 cards in hand per player
    player_hand: HashMap<String, Vec<String>>,
    // remaining deck; pop from this to players hands
    deck: Vec<String>,
    // stage of the game
    stage: RoomStage,
    // round number
    round: u16,
    // order of players being "active"
    player_order: Vec<String>,
    active_player: usize, // index into player_order
    // map to mpsc which sends messages to specific players
    player_to_socket: HashMap<String, mpsc::Sender<ServerMsg>>,
    // cards that have left hands (played or dropped by leaving players)
    discard_pile: Vec<String>,

    /** Round-specific information */
    // chosen description by active player
    current_description: String,
    // for the active player, this is the active card; for other players, this is the card they chose
    player_to_current_card: HashMap<String, String>,
    // for each player, the card they voted for as being the active's card
    // they cannot vote for themselves
    player_to_vote: HashMap<String, String>,
    // configured win condition for this room
    win_condition: WinCondition,
    // increments whenever draw deck is refilled from base deck
    deck_refill_count: u32,
}

// main object representing a game
#[derive(Debug)]
pub struct Room {
    // store all informationa about the room
    state: RwLock<RoomState>,
    // send updates to everyone in the room
    broadcast: broadcast::Sender<ServerMsg>,
    // keep pointer to the base deck for refills
    base_deck: Arc<Vec<String>>,
    // last access in seconds
    last_access: AtomicU64,
}

pub fn get_time_s() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

impl Room {
    pub fn new(
        room_id: &str,
        base_deck: Arc<Vec<String>>,
        win_condition: WinCondition,
        creator: Option<String>,
    ) -> Self {
        let state = RoomState {
            room_id: room_id.to_string(),
            creator,
            moderators: HashSet::new(),
            no_connected_moderator_since_s: None,
            removed_players: HashSet::new(),
            players: HashMap::new(),
            deck: base_deck.to_vec(),
            stage: RoomStage::Joining,
            player_order: Vec::new(),
            player_hand: HashMap::new(),
            player_to_socket: HashMap::new(),
            discard_pile: Vec::new(),
            active_player: 0,
            current_description: "".to_string(),
            player_to_current_card: HashMap::new(),
            player_to_vote: HashMap::new(),
            round: 0,
            win_condition,
            deck_refill_count: 0,
        };

        let (tx, _) = broadcast::channel(10);

        Self {
            state: RwLock::new(state),
            broadcast: tx,
            base_deck,
            last_access: AtomicU64::new(get_time_s()),
        }
    }

    fn is_creator(&self, state: &RwLockWriteGuard<RoomState>, name: &str) -> bool {
        state.creator.as_deref() == Some(name)
    }

    fn is_moderator(&self, state: &RwLockWriteGuard<RoomState>, name: &str) -> bool {
        state.players.contains_key(name) && state.moderators.contains(name)
    }

    fn clean_moderators(&self, state: &mut RwLockWriteGuard<RoomState>) {
        let active_players = state.players.keys().cloned().collect::<HashSet<String>>();
        state
            .moderators
            .retain(|name| active_players.contains(name));

        if let Some(creator) = state.creator.clone() {
            if active_players.contains(&creator) {
                state.moderators.insert(creator);
            }
        }
    }

    fn has_connected_moderator(&self, state: &RwLockWriteGuard<RoomState>) -> bool {
        state.moderators.iter().any(|name| {
            state
                .players
                .get(name)
                .map(|player| player.connected)
                .unwrap_or(false)
        })
    }

    fn maybe_promote_moderator(&self, state: &mut RwLockWriteGuard<RoomState>) -> bool {
        self.clean_moderators(state);

        if self.has_connected_moderator(state) {
            state.no_connected_moderator_since_s = None;
            return false;
        }

        let now = get_time_s();
        let since = state.no_connected_moderator_since_s.get_or_insert(now);
        if now.saturating_sub(*since) < MODERATOR_ABSENCE_PROMOTION_DELAY_S {
            return false;
        }

        let candidates: Vec<String> = state
            .players
            .iter()
            .filter(|(_, player)| player.connected)
            .map(|(name, _)| name.clone())
            .filter(|name| !state.moderators.contains(name))
            .collect();

        if candidates.is_empty() {
            return false;
        }

        let mut rng = rand::thread_rng();
        let promoted = candidates.choose(&mut rng).unwrap().to_string();
        state.moderators.insert(promoted);
        state.no_connected_moderator_since_s = None;
        true
    }

    async fn remove_player(
        &self,
        state: &mut RwLockWriteGuard<'_, RoomState>,
        player_name: &str,
        personal_msg: Option<ServerMsg>,
    ) -> Result<bool> {
        if !state.players.contains_key(player_name) {
            return Ok(false);
        }

        let mut moved_cards = HashSet::new();
        if let Some(hand) = state.player_hand.remove(player_name) {
            for card in hand {
                if moved_cards.insert(card.clone()) {
                    state.discard_pile.push(card);
                }
            }
        }

        if let Some(card) = state.player_to_current_card.remove(player_name) {
            if moved_cards.insert(card.clone()) {
                state.discard_pile.push(card);
            }
        }

        state.player_to_vote.remove(player_name);

        if let Some(pos) = state
            .player_order
            .iter()
            .position(|player| player == player_name)
        {
            state.player_order.remove(pos);
            if pos < state.active_player && state.active_player > 0 {
                state.active_player -= 1;
            }
            if state.active_player >= state.player_order.len() {
                state.active_player = 0;
            }
        }

        state.players.remove(player_name);
        state.moderators.remove(player_name);
        state.removed_players.insert(player_name.to_string());

        if let Some(tx) = state.player_to_socket.remove(player_name) {
            if let Some(msg) = personal_msg {
                let _ = tx.send(msg.into()).await;
            }
        }

        self.clean_moderators(state);
        Ok(true)
    }

    async fn after_player_removed(
        &self,
        state: &mut RwLockWriteGuard<'_, RoomState>,
    ) -> Result<()> {
        if matches!(state.stage, RoomStage::Joining) {
            self.broadcast_msg(self.room_state(state))?;
            return Ok(());
        }

        if state.players.len() < 3 {
            state.stage = RoomStage::End;
            self.broadcast_msg(ServerMsg::EndGame {})?;
            self.broadcast_msg(self.room_state(state))?;
            return Ok(());
        }

        if matches!(state.stage, RoomStage::End) {
            self.broadcast_msg(self.room_state(state))?;
            return Ok(());
        }

        if matches!(state.stage, RoomStage::Results) {
            if self.should_end_game(state) {
                state.stage = RoomStage::End;
                self.broadcast_msg(ServerMsg::EndGame {})?;
                self.broadcast_msg(self.room_state(state))?;
                return Ok(());
            }

            if state.players.values().all(|player| player.ready) {
                self.init_round(state).await?;
                return Ok(());
            }

            self.broadcast_msg(self.room_state(state))?;
            return Ok(());
        }

        self.init_round(state).await?;
        Ok(())
    }

    fn get_msg(
        &self,
        name: Option<&str>,
        state: &RwLockWriteGuard<RoomState>,
    ) -> Result<ServerMsg> {
        match state.stage {
            RoomStage::ActiveChooses => Ok(ServerMsg::StartRound {
                hand: state.player_hand[name.ok_or_else(|| anyhow!("No name provided"))?].clone(),
            }),
            RoomStage::PlayersChoose => Ok(ServerMsg::PlayersChoose {
                description: state.current_description.clone(),
                hand: state.player_hand[name.ok_or_else(|| anyhow!("No name provided"))?].clone(),
            }),
            RoomStage::Voting => Ok(ServerMsg::BeginVoting {
                center_cards: self.get_center_cards(state),
                description: state.current_description.clone(),
                disabled_card: name.and_then(|player_name| {
                    if player_name == state.player_order[state.active_player].as_str() {
                        None
                    } else {
                        state.player_to_current_card.get(player_name).cloned()
                    }
                }),
            }),
            RoomStage::Results => Ok(ServerMsg::Results {
                player_to_vote: state.player_to_vote.clone(),
                player_to_current_card: state.player_to_current_card.clone(),
                active_card: state
                    .player_to_current_card
                    .get(&self.get_active_player(state)?)
                    .unwrap()
                    .to_string(),
                point_change: self.compute_results(state),
            }),
            RoomStage::End => Ok(ServerMsg::EndGame {}),
            _ => Err(anyhow!("No msg to send")),
        }
    }

    fn get_center_cards(&self, state: &RwLockWriteGuard<RoomState>) -> Vec<String> {
        let mut center_cards: Vec<String> = state
            .player_to_current_card
            .values()
            .map(|e| e.to_string())
            .collect();
        center_cards.shuffle(&mut rand::thread_rng());
        center_cards
    }

    fn get_active_player(&self, state: &RwLockWriteGuard<RoomState>) -> Result<String> {
        if matches!(state.stage, RoomStage::Joining) {
            return Err(anyhow!("Failed to find active player"));
        }

        Ok(state
            .player_order
            .get(state.active_player)
            .unwrap() // unreachable
            .to_string())
    }

    async fn init_voting(&self, state: &mut RwLockWriteGuard<'_, RoomState>) -> Result<()> {
        state.stage = RoomStage::Voting;

        // choose random card for those who didn't choose by the deadline
        for player in state.player_order.clone().iter() {
            if !state.player_to_current_card.contains_key(player) {
                let mut rng = rand::thread_rng();
                let card = state.player_hand[player].choose(&mut rng).unwrap().clone();
                state
                    .player_to_current_card
                    .insert(player.to_string(), card);
            }
        }

        self.clear_ready(state);

        // remove cards from hand that were put in the center
        for (player, card) in state.player_to_current_card.clone().iter() {
            if let Some(hand) = state.player_hand.get_mut(player) {
                if let Some(pos) = hand.iter().position(|e| e == card) {
                    let played_card = hand.remove(pos);
                    state.discard_pile.push(played_card);
                }
            }
        }

        for player in state.player_order.iter() {
            let player_name = player.as_str();
            let _ = self
                .send_msg(state, player_name, self.get_msg(Some(player_name), state)?)
                .await;
        }
        self.broadcast_msg(self.room_state(&state))?;

        Ok(())
    }

    fn init_results(&self, state: &mut RwLockWriteGuard<RoomState>) -> Result<()> {
        state.stage = RoomStage::Results;

        let center_cards = self.get_center_cards(state);

        // choose random card to vote for if the player didn't choose
        for player in state.player_order.clone().iter() {
            if player != &state.player_order[state.active_player]
                && !state.player_to_vote.contains_key(player)
            {
                // choose random card
                let mut rng = rand::thread_rng();
                let mut card = center_cards.choose(&mut rng).unwrap().clone();

                // ensure player cannot choose their own card
                while &card == state.player_to_current_card.get(player).unwrap() {
                    card = center_cards.choose(&mut rng).unwrap().clone();
                }

                state.player_to_vote.insert(player.to_string(), card);
            }
        }

        let point_change = self.compute_results(state);

        // update with the point change
        state.players.iter_mut().for_each(|(player, info)| {
            if let Some(points) = point_change.get(player) {
                info.points += points;
            }
        });

        self.clear_ready(state);

        // send results to everyone
        self.broadcast_msg(self.get_msg(None, &state)?)?;
        self.broadcast_msg(self.room_state(&state))?;

        Ok(())
    }

    fn check_deck(&self, state: &mut RwLockWriteGuard<'_, RoomState>) -> bool {
        if state.deck.len() < state.player_order.len() {
            if matches!(state.win_condition, WinCondition::CardsFinish) {
                return false;
            }

            if !state.discard_pile.is_empty() {
                let mut refill = std::mem::take(&mut state.discard_pile);
                refill.shuffle(&mut rand::thread_rng());
                state.deck.append(&mut refill);
            } else {
                // fallback for older rooms where discard wasn't tracked
                let mut new_deck = self.base_deck.to_vec();

                // get all cards currently in hands
                let mut all_hands = Vec::new();
                for player in state.player_hand.keys() {
                    all_hands.append(&mut state.player_hand[player].clone());
                }

                // remove cards from deck
                for card in all_hands {
                    if let Some(pos) = new_deck.iter().position(|e| e == &card) {
                        new_deck.remove(pos);
                    }
                }

                state.deck = new_deck;
                state.deck.shuffle(&mut rand::thread_rng());
            }

            state.deck_refill_count += 1;
            return true;
        }

        false
    }

    async fn init_round(&self, state: &mut RwLockWriteGuard<'_, RoomState>) -> Result<()> {
        if state.players.len() < 3 {
            return Err(anyhow!("Not enough players"));
        }

        let is_first_round = state.round == 0;
        let next_active_player = if is_first_round {
            0
        } else {
            (state.active_player + 1) % state.player_order.len()
        };

        // finalize players
        if is_first_round {
            // first round
            state.active_player = 0;
            state.player_order = state.players.keys().cloned().collect::<Vec<_>>();
            state.player_order.shuffle(&mut rand::thread_rng());
        } else {
            // not enough cards, reload
            self.check_deck(state);
        }

        // shuffle deck
        state.deck.shuffle(&mut rand::thread_rng());

        // clear current chosen cards
        state.player_to_current_card.clear();
        state.player_to_vote.clear();

        // ensure all players have 6 cards
        let mut player_hand = state.player_hand.clone();

        let mut deck = state.deck.clone();
        for player in state.players.keys() {
            if !player_hand.contains_key(player) {
                player_hand.insert(player.clone(), Vec::new());
            }

            while player_hand.get(player).unwrap().len() < 6 {
                let next_card = match deck.pop() {
                    Some(card) => card,
                    None => {
                        if matches!(state.win_condition, WinCondition::CardsFinish) {
                            state.stage = RoomStage::End;
                            self.broadcast_msg(ServerMsg::EndGame {})?;
                            return Ok(());
                        }
                        return Err(anyhow!("Not enough cards in the deck"));
                    }
                };
                player_hand.get_mut(player).unwrap().push(next_card);
            }
        }

        state.round += 1;
        state.active_player = next_active_player;
        state.deck = deck;
        state.player_hand = player_hand;
        state.stage = RoomStage::ActiveChooses;

        // notify players of the game start and their hands
        for player in state.player_order.iter() {
            let _ = self
                .send_msg(&state, &player, self.get_msg(Some(&player), &state)?)
                .await;
        }

        self.clear_ready(state);
        self.broadcast_msg(self.room_state(&state))?;

        Ok(())
    }

    pub async fn handle_client_msg(&self, name: &str, msg: WsMessage) -> Result<()> {
        let mut state = self.state.write().await;

        let msg: ClientMsg = serde_json::from_str(msg.to_text()?)
            .context(format!("Failed to deserialize client msg: {:?}", msg))?;

        println!("Handling client message: {:?}", msg);

        if self.maybe_promote_moderator(&mut state) {
            self.broadcast_msg(self.room_state(&state))?;
        }

        if !matches!(msg, ClientMsg::Ping {}) && !state.players.contains_key(name) {
            return Ok(());
        }

        match msg {
            ClientMsg::Ready {} => {
                if matches!(state.stage, RoomStage::Joining) {
                    let can_start = self.is_moderator(&state, name);
                    if !can_start {
                        return Ok(());
                    }

                    if state.players.len() < 3 {
                        if let Some(tx) = state.player_to_socket.get(name) {
                            tx.send(
                                ServerMsg::ErrorMsg("Need at least 3 players".to_string()).into(),
                            )
                            .await?;
                        }
                        return Ok(());
                    }

                    self.init_round(&mut state).await?;
                    return Ok(());
                }

                if matches!(state.stage, RoomStage::Results) {
                    state
                        .players
                        .get_mut(name)
                        .ok_or_else(|| anyhow!("Unreachable: cannot ready player {}", name))?
                        .ready = true;

                    self.broadcast_msg(self.room_state(&state))?;

                    if self.should_end_game(&state) {
                        state.stage = RoomStage::End;
                        self.broadcast_msg(self.get_msg(None, &state)?)?;
                        return Ok(());
                    }

                    // check if everyone is ready for next round
                    if state.players.values().filter(|p| p.ready).count() == state.players.len() {
                        if state.players.len() >= 3 {
                            self.init_round(&mut state).await?;
                        } else {
                            self.broadcast_msg(
                                ServerMsg::ErrorMsg("Need at least 3 players".to_string()).into(),
                            )?;
                        }
                    }
                }
            }
            ClientMsg::StartGame {} => {
                if !matches!(state.stage, RoomStage::Joining) {
                    return Ok(());
                }

                if !self.is_moderator(&state, name) {
                    if let Some(tx) = state.player_to_socket.get(name) {
                        tx.send(
                            ServerMsg::ErrorMsg("Only moderators can start".to_string()).into(),
                        )
                        .await?;
                    }
                    return Ok(());
                }

                if state.players.len() < 3 {
                    if let Some(tx) = state.player_to_socket.get(name) {
                        tx.send(ServerMsg::ErrorMsg("Need at least 3 players".to_string()).into())
                            .await?;
                    }
                    return Ok(());
                }

                self.init_round(&mut state).await?;
            }
            ClientMsg::LeaveRoom {} => {
                let removed = self
                    .remove_player(
                        &mut state,
                        name,
                        Some(ServerMsg::LeftRoom {
                            reason: "You left the game".to_string(),
                        }),
                    )
                    .await?;
                if removed {
                    self.after_player_removed(&mut state).await?;
                }
            }
            ClientMsg::KickPlayer { player } => {
                if !self.is_moderator(&state, name) {
                    if let Some(tx) = state.player_to_socket.get(name) {
                        tx.send(
                            ServerMsg::ErrorMsg("Only moderators can kick players".to_string())
                                .into(),
                        )
                        .await?;
                    }
                    return Ok(());
                }

                let target = player.trim();
                if target.is_empty() {
                    return Ok(());
                }

                if state.creator.as_deref() == Some(target) {
                    if let Some(tx) = state.player_to_socket.get(name) {
                        tx.send(ServerMsg::ErrorMsg("Creator cannot be kicked".to_string()).into())
                            .await?;
                    }
                    return Ok(());
                }

                let removed = self
                    .remove_player(
                        &mut state,
                        target,
                        Some(ServerMsg::Kicked {
                            reason: "You were kicked from the game".to_string(),
                        }),
                    )
                    .await?;
                if removed {
                    self.after_player_removed(&mut state).await?;
                }
            }
            ClientMsg::SetModerator { player, enabled } => {
                let is_creator = self.is_creator(&state, name);
                let is_moderator = self.is_moderator(&state, name);

                if !is_creator && !is_moderator {
                    if let Some(tx) = state.player_to_socket.get(name) {
                        tx.send(
                            ServerMsg::ErrorMsg(
                                "Only moderators can promote moderators".to_string(),
                            )
                            .into(),
                        )
                        .await?;
                    }
                    return Ok(());
                }

                let target = player.trim();
                if target.is_empty() || !state.players.contains_key(target) {
                    return Ok(());
                }

                if !enabled && !is_creator {
                    if let Some(tx) = state.player_to_socket.get(name) {
                        tx.send(
                            ServerMsg::ErrorMsg(
                                "Only the creator can demote moderators".to_string(),
                            )
                            .into(),
                        )
                        .await?;
                    }
                    return Ok(());
                }

                if state.creator.as_deref() == Some(target) && !enabled {
                    if let Some(tx) = state.player_to_socket.get(name) {
                        tx.send(
                            ServerMsg::ErrorMsg("Creator must remain a moderator".to_string())
                                .into(),
                        )
                        .await?;
                    }
                    return Ok(());
                }

                if enabled {
                    state.moderators.insert(target.to_string());
                } else {
                    state.moderators.remove(target);
                }
                self.clean_moderators(&mut state);
                self.broadcast_msg(self.room_state(&state))?;
            }
            ClientMsg::ActivePlayerChooseCard { card, description } => {
                if matches!(state.stage, RoomStage::ActiveChooses)
                    && !state.player_order.is_empty()
                    && state.player_order[state.active_player] == name
                {
                    // verify that player has this card
                    if !state
                        .player_hand
                        .get(name)
                        .map(|cards| cards.contains(&card))
                        .unwrap_or(false)
                    {
                        return Err(anyhow!("Invalid card chosen by active player"));
                    }

                    let description = description.trim();
                    // verify that the description is not empty and is one word
                    if description.is_empty() {
                        if let Some(tx) = state.player_to_socket.get(name) {
                            tx.send(
                                ServerMsg::ErrorMsg("Description must not be empty".to_string())
                                    .into(),
                            )
                            .await?;
                        }
                        return Ok(());
                    }
                    state.current_description = description.to_string();
                    state.stage = RoomStage::PlayersChoose;

                    // record choice
                    state
                        .player_to_current_card
                        .insert(name.to_string(), card.to_string());

                    // notify players of the active player's choice
                    for player in state.player_order.iter() {
                        let _ = self
                            .send_msg(&state, &player, self.get_msg(Some(&player), &state)?)
                            .await;
                    }

                    self.clear_ready(&mut state);
                    self.broadcast_msg(self.room_state(&state))?;
                }
            }
            ClientMsg::PlayerChooseCard { card } => {
                if matches!(state.stage, RoomStage::PlayersChoose) {
                    if state.player_order.is_empty() {
                        return Ok(());
                    }
                    if state.player_order[state.active_player] != name {
                        // verify that player has this card
                        if !state
                            .player_hand
                            .get(name)
                            .map(|cards| cards.contains(&card))
                            .unwrap_or(false)
                        {
                            return Err(anyhow!("Invalid card chosen by player"));
                        }

                        // record choice
                        state
                            .player_to_current_card
                            .insert(name.to_string(), card.to_string());

                        // ready
                        state.players.get_mut(name).unwrap().ready = true;
                        self.broadcast_msg(self.room_state(&state))?;

                        // check if everyone except for the active player is ready
                        if state.players.values().filter(|p| p.ready).count()
                            == state.players.len() - 1
                        {
                            self.init_voting(&mut state).await?;
                        }
                    }
                }
            }
            ClientMsg::Vote { card } => {
                if matches!(state.stage, RoomStage::Voting) {
                    if state.player_order.is_empty() {
                        return Ok(());
                    }
                    // verify that the player is not the active player
                    if state.player_order[state.active_player] == name {
                        println!(
                            "{} is the active player",
                            state.player_order[state.active_player]
                        );
                        println!("{} is trying to vote", name);
                        return Err(anyhow!("Active player cannot vote"));
                    }

                    // verify that the card is in the center
                    if !state.player_to_current_card.values().any(|e| e == &card) {
                        return Err(anyhow!("Invalid card"));
                    }

                    // verify that this player is not voting for their own code or send an error message
                    if state.player_to_current_card.get(name).map(|v| v == &card) == Some(true) {
                        if let Some(socket) = state.player_to_socket.get(name) {
                            socket
                                .send(
                                    ServerMsg::ErrorMsg(
                                        "You cannot vote for your own card".to_string(),
                                    )
                                    .into(),
                                )
                                .await?;
                        }
                        return Ok(());
                    }

                    // record vote
                    state
                        .player_to_vote
                        .insert(name.to_string(), card.to_string());

                    // ready
                    state.players.get_mut(name).unwrap().ready = true;
                    self.broadcast_msg(self.room_state(&state))?;

                    // check if everyone except for the active player is ready
                    if state.players.values().filter(|p| p.ready).count() == state.players.len() - 1
                    {
                        self.init_results(&mut state)?;
                    }
                }
            }
            _ => {
                // nothing
            }
        }

        Ok(())
    }

    fn compute_results(&self, state: &RwLockWriteGuard<RoomState>) -> HashMap<String, u16> {
        let mut point_change: HashMap<String, u16> = HashMap::new();
        let active_player = state.player_order[state.active_player].clone();
        let active_card = state
            .player_to_current_card
            .get(&active_player)
            .unwrap()
            .clone();

        let mut votes_for_card: HashMap<String, u16> = HashMap::new();

        for (_, card) in state.player_to_vote.iter() {
            *votes_for_card.entry(card.to_string()).or_insert(0) += 1;
        }

        let votes_for_active_card = *votes_for_card.get(&active_card).unwrap_or(&0);
        if votes_for_active_card == 0 {
            // nobody voted for active card
            for (player, _) in state.player_to_vote.iter() {
                point_change.insert(player.to_string(), 2);
            }

            for (player, card) in state.player_to_current_card.iter() {
                if player != &active_player {
                    *point_change.get_mut(player).unwrap() +=
                        votes_for_card.get(card).unwrap_or(&0);
                }
            }

            point_change.insert(active_player.clone(), 0);
        } else if votes_for_active_card == (state.player_order.len() - 1) as u16 {
            // everyone voted for active card
            for (player, _) in state.player_to_vote.iter() {
                point_change.insert(player.to_string(), 2);
            }
            point_change.insert(active_player.clone(), 0);
        } else {
            // someone voted for the active card
            for (player, card) in state.player_to_vote.iter() {
                if card == &active_card {
                    point_change.insert(player.to_string(), 3);
                } else {
                    point_change.insert(player.to_string(), 0);
                }
            }

            for (player, card) in state.player_to_current_card.iter() {
                if player != &active_player {
                    *point_change.get_mut(player).unwrap() +=
                        votes_for_card.get(card).unwrap_or(&0);
                }
            }

            point_change.insert(active_player.clone(), 3);
        }

        point_change
    }

    fn should_end_game(&self, state: &RwLockWriteGuard<RoomState>) -> bool {
        match state.win_condition {
            WinCondition::Points { target_points } => {
                let max_points = state
                    .players
                    .values()
                    .map(|p| p.points)
                    .max()
                    .unwrap_or_default();
                max_points >= target_points
            }
            WinCondition::Cycles { target_cycles } => {
                if state.round == 0 {
                    return false;
                }

                let players_per_cycle = if !state.player_order.is_empty() {
                    state.player_order.len()
                } else {
                    state.players.len()
                };

                if players_per_cycle == 0 {
                    return false;
                }

                let required_rounds = u32::from(target_cycles) * players_per_cycle as u32;
                u32::from(state.round) >= required_rounds
            }
            WinCondition::CardsFinish => false,
        }
    }

    pub async fn on_connection(&self, socket: &mut WebSocket, name: &str) {
        // public funciton
        if let Err(e) = self.attempt_join(socket, name).await {
            println!("Error in attempt_join: {:?}", e);
            return;
        }

        let res = self.run_ws_loop(socket, name).await;
        println!("Player {} has left", name);

        self.last_access.store(get_time_s(), Ordering::Relaxed);
        let mut state = self.state.write().await;

        if state.removed_players.remove(name) {
            // already removed explicitly (leave/kick)
        } else if matches!(state.stage, RoomStage::Joining) {
            state.players.remove(name);
            state.moderators.remove(name);
        } else {
            if let Some(player) = state.players.get_mut(name) {
                player.connected = false;
            }
        }

        state.player_to_socket.remove(name);
        self.clean_moderators(&mut state);
        self.maybe_promote_moderator(&mut state);

        if let Err(e) = res {
            println!("Error in run_ws_loop: {:?}", e);
        }

        if let Err(e) = self.broadcast_msg(self.room_state(&state)) {
            println!("Error sending broadcast: {}", e);
        }
    }

    async fn attempt_join(&self, socket: &mut WebSocket, name: &str) -> Result<()> {
        if name.is_empty() {
            socket
                .send(ServerMsg::ErrorMsg("Name cannot be empty".to_string()).into())
                .await?;
            return Err(anyhow!("Name cannot be empty"));
        }

        println!("Handling join for {}", name);

        let mut state = self.state.write().await;

        if let Some(player) = state.players.get_mut(name) {
            // player already exists in the game
            // and not in joining anymore
            // if in joining then player.active will be true

            if !player.connected {
                player.connected = true;
            } else {
                socket
                    .send(ServerMsg::ErrorMsg("Name already taken".to_string()).into())
                    .await?;
                return Err(anyhow!("Name already taken"));
            }
        } else if matches!(state.stage, RoomStage::Joining) {
            // still in joining and not yet joined
            if state.players.len() < 8 {
                if state.creator.is_none() {
                    state.creator = Some(name.to_string());
                }

                state.players.insert(
                    name.to_string(),
                    PlayerInfo {
                        connected: true,
                        points: 0,
                        ready: true,
                    },
                );
            } else {
                socket
                    .send(ServerMsg::ErrorMsg("Too many players!".to_string()).into())
                    .await?;
                return Err(anyhow!("Too many players!"));
            }
        } else {
            socket
                .send(ServerMsg::ErrorMsg("Game has already started".to_string()).into())
                .await?;
            return Err(anyhow!("Game has already started"));
        }

        if state.creator.as_deref() == Some(name) {
            state.moderators.insert(name.to_string());
        }
        self.clean_moderators(&mut state);
        self.maybe_promote_moderator(&mut state);

        self.broadcast_msg(self.room_state(&state).into())?; // will not receive this one yet
        socket.send(self.room_state(&state).into()).await?;
        if let Ok(msg) = self.get_msg(Some(name), &state) {
            socket.send(msg.into()).await?;
        }

        Ok(())
    }

    async fn run_ws_loop(&self, socket: &mut WebSocket, name: &str) -> Result<()> {
        let (tx, mut rx) = mpsc::channel(10);
        self.state
            .write()
            .await
            .player_to_socket
            .insert(name.to_string(), tx);
        let mut broadcast_updates = self.broadcast.subscribe();

        loop {
            tokio::select! {
                msg = broadcast_updates.recv() => {
                    socket.send(msg?.into()).await?;
                }
                msg = socket.recv() => {
                    match msg {
                        Some(Ok(msg)) => {
                            self.handle_client_msg(name, msg).await?;
                        }
                        _ => break
                    }
                },
                msg = rx.recv() => {
                    match msg {
                        Some(msg) => {
                            socket.send(msg.into()).await?;
                        }
                        _ => break,
                    }
                }
            }
        }

        Ok(())
    }

    fn broadcast_msg(&self, msg: ServerMsg) -> Result<()> {
        if self.broadcast.receiver_count() != 0 {
            self.broadcast.send(msg)?;
        }
        Ok(())
    }

    async fn send_msg(
        &self,
        state: &RwLockWriteGuard<'_, RoomState>,
        name: &str,
        msg: ServerMsg,
    ) -> Result<()> {
        let socket = state.player_to_socket.get(name).ok_or_else(|| {
            println!("Cannot find socket for {}", name);
            anyhow!("Cannot find socket for {}", name)
        })?;

        socket.send(msg.into()).await?;
        Ok(())
    }

    fn clear_ready(&self, state: &mut RwLockWriteGuard<RoomState>) {
        for (_, player) in state.players.iter_mut() {
            player.ready = false;
        }
    }

    pub async fn run_maintenance(&self) {
        let mut state = self.state.write().await;
        if self.maybe_promote_moderator(&mut state) {
            let _ = self.broadcast_msg(self.room_state(&state));
        }
    }

    pub fn num_active(&self) -> usize {
        self.broadcast.receiver_count()
    }

    pub fn last_access(&self) -> u64 {
        self.last_access.load(Ordering::Relaxed)
    }

    pub async fn get_room_state(&self) -> ServerMsg {
        let state = self.state.write().await;
        self.room_state(&state)
    }

    fn room_state(&self, state: &RwLockWriteGuard<RoomState>) -> ServerMsg {
        let mut moderators = state.moderators.iter().cloned().collect::<Vec<_>>();
        moderators.sort();

        ServerMsg::RoomState {
            room_id: state.room_id.clone(),
            players: state.players.clone(),
            creator: state.creator.clone(),
            moderators,
            stage: state.stage,
            active_player: state.player_order.get(state.active_player).cloned(),
            player_order: state.player_order.clone(),
            round: state.round,
            cards_remaining: state.deck.len() as u32,
            deck_refill_count: state.deck_refill_count,
            win_condition: state.win_condition,
        }
    }
}
