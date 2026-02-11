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
        observers: HashMap<String, ObserverInfo>,
        creator: Option<String>,
        moderators: Vec<String>,
        stage: RoomStage,
        active_player: Option<String>,
        player_order: Vec<String>,
        round: u16,
        cards_remaining: u32,
        deck_refill_count: u32,
        win_condition: WinCondition,
        allow_new_players_midgame: bool,
        paused_reason: Option<String>,
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

#[derive(Debug, Serialize, Deserialize)]
pub enum ClientMsg {
    Ready {},
    StartGame {},
    LeaveRoom {},
    KickPlayer {
        player: String,
    },
    SetModerator {
        player: String,
        enabled: bool,
    },
    SetObserver {
        player: String,
        enabled: bool,
    },
    SetAllowMidgameJoin {
        enabled: bool,
    },
    ResumeGame {},
    RequestJoinFromObserver {},
    JoinRoom {
        room_id: String,
        name: String,
        token: String,
    },
    CreateRoom {
        name: String,
    },
    ActivePlayerChooseCard {
        card: String,
        description: String,
    },
    PlayerChooseCard {
        card: String,
    },
    Vote {
        card: String,
    },
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
    // game is paused due to low amount of players
    Paused,
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

#[derive(Debug, Serialize, Clone)]
pub struct ObserverInfo {
    connected: bool,
    points: u16,
    join_requested: bool,
    auto_join_on_next_round: bool,
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
    // members removed from this room explicitly (leave/kick)
    removed_players: HashSet<String>,
    // observers are room members but not active players in a round
    observers: HashMap<String, ObserverInfo>,
    // stable per-name token for reconnect/auth
    name_tokens: HashMap<String, String>,
    // active connection generation for each member
    connection_generation: HashMap<String, u64>,
    next_generation: u64,
    // moderators can toggle this after the game starts
    allow_new_players_midgame: bool,
    // user-facing pause reason for RoomStage::Paused
    paused_reason: Option<String>,
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
    // cap for players + observers in a room
    max_members: usize,
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
        max_members: usize,
    ) -> Self {
        let state = RoomState {
            room_id: room_id.to_string(),
            creator,
            moderators: HashSet::new(),
            no_connected_moderator_since_s: None,
            removed_players: HashSet::new(),
            observers: HashMap::new(),
            name_tokens: HashMap::new(),
            connection_generation: HashMap::new(),
            next_generation: 0,
            allow_new_players_midgame: true,
            paused_reason: None,
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
            max_members,
            last_access: AtomicU64::new(get_time_s()),
        }
    }

    fn is_creator(&self, state: &RwLockWriteGuard<RoomState>, name: &str) -> bool {
        state.creator.as_deref() == Some(name)
    }

    fn member_exists(&self, state: &RwLockWriteGuard<RoomState>, name: &str) -> bool {
        state.players.contains_key(name) || state.observers.contains_key(name)
    }

    fn is_moderator(&self, state: &RwLockWriteGuard<RoomState>, name: &str) -> bool {
        self.member_exists(state, name) && state.moderators.contains(name)
    }

    fn clean_moderators(&self, state: &mut RwLockWriteGuard<RoomState>) {
        let active_members = state
            .players
            .keys()
            .chain(state.observers.keys())
            .cloned()
            .collect::<HashSet<String>>();
        state
            .moderators
            .retain(|name| active_members.contains(name));

        if let Some(creator) = state.creator.clone() {
            if active_members.contains(&creator) {
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
                .or_else(|| state.observers.get(name).map(|observer| observer.connected))
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
            .chain(
                state
                    .observers
                    .iter()
                    .filter(|(_, observer)| observer.connected)
                    .map(|(name, _)| name.clone()),
            )
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

    fn next_connection_generation(&self, state: &mut RwLockWriteGuard<'_, RoomState>) -> u64 {
        state.next_generation = state.next_generation.saturating_add(1);
        state.next_generation
    }

    fn total_members(&self, state: &RwLockWriteGuard<RoomState>) -> usize {
        state.players.len() + state.observers.len()
    }

    fn non_observer_player_count(&self, state: &RwLockWriteGuard<RoomState>) -> usize {
        state.players.len()
    }

    fn disconnect_previous_session(
        &self,
        state: &mut RwLockWriteGuard<'_, RoomState>,
        name: &str,
    ) -> Result<()> {
        if let Some(tx) = state.player_to_socket.get(name) {
            let _ = tx.send(
                ServerMsg::LeftRoom {
                    reason: "You signed in from another session".to_string(),
                }
                .into(),
            );
        }
        state.player_to_socket.remove(name);
        Ok(())
    }

    fn can_join_midgame(&self, state: &RwLockWriteGuard<RoomState>) -> bool {
        state.allow_new_players_midgame
    }

    fn has_valid_token_for_name(
        &self,
        state: &RwLockWriteGuard<RoomState>,
        name: &str,
        token: &str,
    ) -> bool {
        match state.name_tokens.get(name) {
            Some(existing_token) => existing_token == token,
            None => true,
        }
    }

    fn pause_reason(&self) -> String {
        "Need at least 3 non-observer players. Ask a player to join.".to_string()
    }

    fn reset_round_keep_hands(&self, state: &mut RwLockWriteGuard<'_, RoomState>) {
        state.current_description.clear();
        state.player_to_current_card.clear();
        state.player_to_vote.clear();
        self.clear_ready(state);
    }

    fn maybe_pause_for_low_player_count(
        &self,
        state: &mut RwLockWriteGuard<'_, RoomState>,
    ) -> Result<bool> {
        if matches!(state.stage, RoomStage::Joining | RoomStage::End) {
            return Ok(false);
        }

        if self.non_observer_player_count(state) >= 3 {
            return Ok(false);
        }

        self.reset_round_keep_hands(state);
        state.stage = RoomStage::Paused;
        state.paused_reason = Some(self.pause_reason());
        self.broadcast_msg(self.room_state(state))?;
        Ok(true)
    }

    fn observer_floor_score(&self, state: &RwLockWriteGuard<'_, RoomState>) -> u16 {
        if let Some(min_active) = state.players.values().map(|p| p.points).min() {
            return min_active;
        }

        let all_points = state
            .players
            .values()
            .map(|p| p.points)
            .chain(state.observers.values().map(|o| o.points))
            .collect::<Vec<u16>>();
        all_points.into_iter().min().unwrap_or(0)
    }

    fn sync_player_order_with_players(&self, state: &mut RwLockWriteGuard<'_, RoomState>) {
        let existing = state.players.keys().cloned().collect::<HashSet<_>>();
        state.player_order.retain(|name| existing.contains(name));

        let missing: Vec<String> = state
            .players
            .keys()
            .filter(|name| !state.player_order.contains(name))
            .cloned()
            .collect();
        for name in missing {
            state.player_order.push(name);
        }

        if state.player_order.is_empty() {
            state.active_player = 0;
        } else if state.active_player >= state.player_order.len() {
            state.active_player = 0;
        }
    }

    fn should_promote_observer(&self, observer: &ObserverInfo) -> bool {
        observer.auto_join_on_next_round || observer.join_requested
    }

    fn promote_requested_observers(&self, state: &mut RwLockWriteGuard<'_, RoomState>) -> usize {
        let to_promote: Vec<String> = state
            .observers
            .iter()
            .filter(|(_, info)| self.should_promote_observer(info))
            .map(|(name, _)| name.clone())
            .collect();
        if to_promote.is_empty() {
            return 0;
        }

        let floor = self.observer_floor_score(state);
        let mut promoted = 0usize;
        for name in to_promote {
            if let Some(observer) = state.observers.remove(&name) {
                state.players.insert(
                    name.clone(),
                    PlayerInfo {
                        connected: observer.connected,
                        points: observer.points.max(floor),
                        ready: false,
                    },
                );
                promoted += 1;
            }
        }

        self.clean_moderators(state);
        self.sync_player_order_with_players(state);
        promoted
    }

    async fn convert_player_to_observer(
        &self,
        state: &mut RwLockWriteGuard<'_, RoomState>,
        player_name: &str,
        auto_join_on_next_round: bool,
    ) -> Result<bool> {
        if !state.players.contains_key(player_name) {
            return Ok(false);
        }

        let points = state
            .players
            .get(player_name)
            .map(|p| p.points)
            .unwrap_or(0);
        let connected = state
            .players
            .get(player_name)
            .map(|p| p.connected)
            .unwrap_or(false);

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
        state.observers.insert(
            player_name.to_string(),
            ObserverInfo {
                connected,
                points,
                join_requested: false,
                auto_join_on_next_round,
            },
        );
        self.clean_moderators(state);
        Ok(true)
    }

    async fn remove_observer(
        &self,
        state: &mut RwLockWriteGuard<'_, RoomState>,
        name: &str,
        personal_msg: Option<ServerMsg>,
    ) -> Result<bool> {
        if !state.observers.contains_key(name) {
            return Ok(false);
        }

        state.observers.remove(name);
        state.moderators.remove(name);
        if state.creator.as_deref() == Some(name) {
            state.creator = None;
        }
        state.name_tokens.remove(name);
        state.connection_generation.remove(name);
        state.removed_players.insert(name.to_string());

        if let Some(tx) = state.player_to_socket.remove(name) {
            if let Some(msg) = personal_msg {
                let _ = tx.send(msg.into()).await;
            }
        }

        self.clean_moderators(state);
        Ok(true)
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
        if state.creator.as_deref() == Some(player_name) {
            state.creator = None;
        }
        state.name_tokens.remove(player_name);
        state.connection_generation.remove(player_name);
        state.removed_players.insert(player_name.to_string());

        if let Some(tx) = state.player_to_socket.remove(player_name) {
            if let Some(msg) = personal_msg {
                let _ = tx.send(msg.into()).await;
            }
        }

        self.clean_moderators(state);
        Ok(true)
    }

    async fn restart_round_keep_hands(
        &self,
        state: &mut RwLockWriteGuard<'_, RoomState>,
    ) -> Result<()> {
        if state.player_order.is_empty() {
            self.sync_player_order_with_players(state);
        }
        if state.player_order.is_empty() {
            state.stage = RoomStage::Paused;
            state.paused_reason = Some(self.pause_reason());
            self.broadcast_msg(self.room_state(state))?;
            return Ok(());
        }

        self.reset_round_keep_hands(state);
        state.stage = RoomStage::ActiveChooses;
        state.paused_reason = None;
        for player in state.player_order.clone().iter() {
            let player_name = player.as_str();
            let _ = self
                .send_msg(state, player_name, self.get_msg(Some(player_name), state)?)
                .await;
        }
        self.broadcast_msg(self.room_state(state))?;
        Ok(())
    }

    fn active_player_name<'a>(&self, state: &'a RwLockWriteGuard<RoomState>) -> Option<&'a str> {
        state
            .player_order
            .get(state.active_player)
            .map(|name| name.as_str())
    }

    async fn after_member_removed_or_observered(
        &self,
        state: &mut RwLockWriteGuard<'_, RoomState>,
    ) -> Result<()> {
        if matches!(state.stage, RoomStage::Joining) {
            self.broadcast_msg(self.room_state(state))?;
            return Ok(());
        }

        if matches!(state.stage, RoomStage::End) {
            self.broadcast_msg(self.room_state(state))?;
            return Ok(());
        }

        if self.maybe_pause_for_low_player_count(state)? {
            return Ok(());
        }

        match state.stage {
            RoomStage::Paused => {
                self.broadcast_msg(self.room_state(state))?;
                return Ok(());
            }
            RoomStage::ActiveChooses => {
                // if active player left, restart this round with current hands
                let active_exists = self
                    .active_player_name(state)
                    .map(|name| state.players.contains_key(name))
                    .unwrap_or(false);
                if !active_exists {
                    self.sync_player_order_with_players(state);
                    state.active_player = 0;
                    self.restart_round_keep_hands(state).await?;
                } else {
                    self.broadcast_msg(self.room_state(state))?;
                }
                return Ok(());
            }
            RoomStage::PlayersChoose => {
                let active_exists = self
                    .active_player_name(state)
                    .map(|name| state.players.contains_key(name))
                    .unwrap_or(false);
                if !active_exists {
                    self.sync_player_order_with_players(state);
                    state.active_player = 0;
                    self.restart_round_keep_hands(state).await?;
                    return Ok(());
                }

                let ready_count = state.players.values().filter(|player| player.ready).count();
                if ready_count >= state.players.len().saturating_sub(1) {
                    self.init_voting(state).await?;
                } else {
                    self.broadcast_msg(self.room_state(state))?;
                }
                return Ok(());
            }
            RoomStage::Voting => {
                let active_exists = self
                    .active_player_name(state)
                    .map(|name| {
                        state.players.contains_key(name)
                            && state.player_to_current_card.contains_key(name)
                    })
                    .unwrap_or(false);
                if !active_exists {
                    self.sync_player_order_with_players(state);
                    state.active_player = 0;
                    self.restart_round_keep_hands(state).await?;
                    return Ok(());
                }

                if state.player_to_vote.len() >= state.players.len().saturating_sub(1) {
                    self.init_results(state)?;
                } else {
                    self.broadcast_msg(self.room_state(state))?;
                }
                return Ok(());
            }
            RoomStage::Results => {
                if self.should_end_game(state) {
                    state.stage = RoomStage::End;
                    state.paused_reason = None;
                    self.broadcast_msg(ServerMsg::EndGame {})?;
                    self.broadcast_msg(self.room_state(state))?;
                    return Ok(());
                }

                if state.players.values().all(|player| player.ready) {
                    self.init_round(state).await?;
                } else {
                    self.broadcast_msg(self.room_state(state))?;
                }
                return Ok(());
            }
            _ => {}
        }

        self.broadcast_msg(self.room_state(state))?;
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
            RoomStage::Paused => Err(anyhow!("No stage-specific paused message")),
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
        let _promoted = self.promote_requested_observers(state);

        if self.non_observer_player_count(state) < 3 {
            state.stage = RoomStage::Paused;
            state.paused_reason = Some(self.pause_reason());
            self.broadcast_msg(self.room_state(state))?;
            return Ok(());
        }

        self.sync_player_order_with_players(state);
        if state.player_order.is_empty() {
            return Err(anyhow!("No players available to start round"));
        }

        let is_first_round = state.round == 0;
        let next_active_player = if is_first_round {
            0
        } else {
            (state.active_player + 1) % state.player_order.len()
        };

        if is_first_round {
            state.active_player = 0;
            state.player_order.shuffle(&mut rand::thread_rng());
        } else {
            self.check_deck(state);
        }

        state.deck.shuffle(&mut rand::thread_rng());
        state.player_to_current_card.clear();
        state.player_to_vote.clear();
        state.current_description.clear();
        state.paused_reason = None;

        let mut player_hand = state.player_hand.clone();
        player_hand.retain(|name, _| state.players.contains_key(name));

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

        for player in state.player_order.iter() {
            let _ = self
                .send_msg(&state, &player, self.get_msg(Some(&player), &state)?)
                .await;
        }

        self.clear_ready(state);
        self.broadcast_msg(self.room_state(&state))?;

        Ok(())
    }

    pub async fn handle_client_msg(
        &self,
        name: &str,
        connection_generation: u64,
        msg: WsMessage,
    ) -> Result<()> {
        let mut state = self.state.write().await;

        if state.connection_generation.get(name).copied() != Some(connection_generation) {
            return Ok(());
        }

        let msg: ClientMsg = match msg {
            WsMessage::Text(text) => serde_json::from_str(&text)
                .context(format!("Failed to deserialize client msg: {}", text))?,
            WsMessage::Binary(bytes) => {
                let text = match std::str::from_utf8(bytes.as_ref()) {
                    Ok(text) => text,
                    Err(_) => {
                        println!(
                            "Ignoring non-UTF8 binary client message from {} ({} bytes)",
                            name,
                            bytes.len()
                        );
                        return Ok(());
                    }
                };
                serde_json::from_str(text).context(format!(
                    "Failed to deserialize client msg (binary): {}",
                    text
                ))?
            }
            WsMessage::Ping(_) | WsMessage::Pong(_) | WsMessage::Close(_) => {
                return Ok(());
            }
        };

        println!("Handling client message: {:?}", msg);

        if self.maybe_promote_moderator(&mut state) {
            self.broadcast_msg(self.room_state(&state))?;
        }

        if !matches!(msg, ClientMsg::Ping {}) && !self.member_exists(&state, name) {
            return Ok(());
        }

        match msg {
            ClientMsg::Ready {} => {
                if !state.players.contains_key(name) {
                    return Ok(());
                }

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
                    if state.players.values().filter(|p| p.ready).count() >= state.players.len() {
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
                if !state.players.contains_key(name) {
                    return Ok(());
                }

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
                let removed_player = self
                    .remove_player(
                        &mut state,
                        name,
                        Some(ServerMsg::LeftRoom {
                            reason: "You left the game".to_string(),
                        }),
                    )
                    .await?;
                if removed_player {
                    self.after_member_removed_or_observered(&mut state).await?;
                    return Ok(());
                }

                let removed_observer = self
                    .remove_observer(
                        &mut state,
                        name,
                        Some(ServerMsg::LeftRoom {
                            reason: "You left the game".to_string(),
                        }),
                    )
                    .await?;
                if removed_observer {
                    self.broadcast_msg(self.room_state(&state))?;
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

                let removed_player = self
                    .remove_player(
                        &mut state,
                        target,
                        Some(ServerMsg::Kicked {
                            reason: "You were kicked from the game".to_string(),
                        }),
                    )
                    .await?;
                if removed_player {
                    self.after_member_removed_or_observered(&mut state).await?;
                    return Ok(());
                }

                let removed_observer = self
                    .remove_observer(
                        &mut state,
                        target,
                        Some(ServerMsg::Kicked {
                            reason: "You were kicked from the game".to_string(),
                        }),
                    )
                    .await?;
                if removed_observer {
                    self.broadcast_msg(self.room_state(&state))?;
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
                if target.is_empty() || !self.member_exists(&state, target) {
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
            ClientMsg::SetObserver { player, enabled } => {
                if matches!(state.stage, RoomStage::Joining | RoomStage::End) {
                    return Ok(());
                }

                let target = player.trim();
                if target.is_empty() || !self.member_exists(&state, target) {
                    return Ok(());
                }

                let self_target = target == name;
                if !self_target && !self.is_moderator(&state, name) {
                    if let Some(tx) = state.player_to_socket.get(name) {
                        tx.send(
                            ServerMsg::ErrorMsg(
                                "Only moderators can change other players".to_string(),
                            )
                            .into(),
                        )
                        .await?;
                    }
                    return Ok(());
                }

                if enabled {
                    if !state.players.contains_key(target) {
                        return Ok(());
                    }

                    let target_is_active = matches!(
                        state.stage,
                        RoomStage::ActiveChooses | RoomStage::PlayersChoose | RoomStage::Voting
                    ) && self.active_player_name(&state) == Some(target);
                    if target_is_active {
                        if let Some(tx) = state.player_to_socket.get(name) {
                            tx.send(
                                ServerMsg::ErrorMsg(
                                    "Storyteller cannot become observer this round".to_string(),
                                )
                                .into(),
                            )
                            .await?;
                        }
                        return Ok(());
                    }

                    let converted = self
                        .convert_player_to_observer(&mut state, target, false)
                        .await?;
                    if converted {
                        self.after_member_removed_or_observered(&mut state).await?;
                    }
                } else if let Some(observer) = state.observers.get_mut(target) {
                    observer.join_requested = true;
                    observer.auto_join_on_next_round = false;
                    self.broadcast_msg(self.room_state(&state))?;
                }
            }
            ClientMsg::RequestJoinFromObserver {} => {
                if let Some(observer) = state.observers.get_mut(name) {
                    observer.join_requested = true;
                    observer.auto_join_on_next_round = false;
                    self.broadcast_msg(self.room_state(&state))?;
                }
            }
            ClientMsg::SetAllowMidgameJoin { enabled } => {
                if !self.is_moderator(&state, name) {
                    if let Some(tx) = state.player_to_socket.get(name) {
                        tx.send(
                            ServerMsg::ErrorMsg(
                                "Only moderators can change midgame join settings".to_string(),
                            )
                            .into(),
                        )
                        .await?;
                    }
                    return Ok(());
                }

                if matches!(state.stage, RoomStage::Joining | RoomStage::End) {
                    return Ok(());
                }

                state.allow_new_players_midgame = enabled;
                self.broadcast_msg(self.room_state(&state))?;
            }
            ClientMsg::ResumeGame {} => {
                if !matches!(state.stage, RoomStage::Paused) {
                    return Ok(());
                }

                if !self.is_moderator(&state, name) {
                    if let Some(tx) = state.player_to_socket.get(name) {
                        tx.send(
                            ServerMsg::ErrorMsg("Only moderators can resume the game".to_string())
                                .into(),
                        )
                        .await?;
                    }
                    return Ok(());
                }

                if self.non_observer_player_count(&state) < 3 {
                    if let Some(tx) = state.player_to_socket.get(name) {
                        tx.send(
                            ServerMsg::ErrorMsg(
                                "Need at least 3 non-observer players to resume".to_string(),
                            )
                            .into(),
                        )
                        .await?;
                    }
                    return Ok(());
                }

                self.init_round(&mut state).await?;
            }
            ClientMsg::ActivePlayerChooseCard { card, description } => {
                if !state.players.contains_key(name) {
                    return Ok(());
                }

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
                if !state.players.contains_key(name) {
                    return Ok(());
                }

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
                            >= state.players.len().saturating_sub(1)
                        {
                            self.init_voting(&mut state).await?;
                        }
                    }
                }
            }
            ClientMsg::Vote { card } => {
                if !state.players.contains_key(name) {
                    return Ok(());
                }

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
                    if state.players.values().filter(|p| p.ready).count()
                        >= state.players.len().saturating_sub(1)
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

    pub async fn on_connection(&self, socket: &mut WebSocket, name: &str, token: &str) {
        // public funciton
        let connection_generation = match self.attempt_join(socket, name, token).await {
            Ok(generation) => generation,
            Err(e) => {
                println!("Error in attempt_join: {:?}", e);
                return;
            }
        };

        let res = self.run_ws_loop(socket, name, connection_generation).await;
        println!("Player {} has left", name);

        self.last_access.store(get_time_s(), Ordering::Relaxed);
        let mut state = self.state.write().await;

        if state.connection_generation.get(name).copied() != Some(connection_generation) {
            if let Err(e) = res {
                println!("Error in run_ws_loop (stale session): {:?}", e);
            }
            return;
        }

        if state.removed_players.remove(name) {
            // already removed explicitly (leave/kick)
        } else if matches!(state.stage, RoomStage::Joining) {
            state.players.remove(name);
            state.observers.remove(name);
            state.moderators.remove(name);
            state.name_tokens.remove(name);
            state.connection_generation.remove(name);
        } else {
            if let Some(player) = state.players.get_mut(name) {
                player.connected = false;
            }
            if let Some(observer) = state.observers.get_mut(name) {
                observer.connected = false;
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

    async fn attempt_join(&self, socket: &mut WebSocket, name: &str, token: &str) -> Result<u64> {
        if name.is_empty() {
            socket
                .send(ServerMsg::ErrorMsg("Name cannot be empty".to_string()).into())
                .await?;
            return Err(anyhow!("Name cannot be empty"));
        }
        if token.trim().is_empty() {
            socket
                .send(ServerMsg::ErrorMsg("Token cannot be empty".to_string()).into())
                .await?;
            return Err(anyhow!("Token cannot be empty"));
        }

        println!("Handling join for {}", name);

        let mut state = self.state.write().await;

        if !self.has_valid_token_for_name(&state, name, token) {
            socket
                .send(ServerMsg::ErrorMsg("Name already taken".to_string()).into())
                .await?;
            return Err(anyhow!("Name already taken"));
        }

        let is_known_member = self.member_exists(&state, name);

        if let Some(player) = state.players.get_mut(name) {
            player.connected = true;
            self.disconnect_previous_session(&mut state, name)?;
        } else if let Some(observer) = state.observers.get_mut(name) {
            observer.connected = true;
            self.disconnect_previous_session(&mut state, name)?;
        } else {
            if self.total_members(&state) >= self.max_members {
                socket
                    .send(ServerMsg::ErrorMsg("Room is full".to_string()).into())
                    .await?;
                return Err(anyhow!("Room is full"));
            }

            if !matches!(state.stage, RoomStage::Joining) && !self.can_join_midgame(&state) {
                socket
                    .send(
                        ServerMsg::ErrorMsg("New players are disabled for this game".to_string())
                            .into(),
                    )
                    .await?;
                return Err(anyhow!("New players are disabled"));
            }

            match state.stage {
                RoomStage::Joining => {
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
                }
                RoomStage::ActiveChooses | RoomStage::PlayersChoose | RoomStage::Paused => {
                    state.players.insert(
                        name.to_string(),
                        PlayerInfo {
                            connected: true,
                            points: 0,
                            ready: false,
                        },
                    );
                    state.player_hand.insert(name.to_string(), Vec::new());
                    while state.player_hand.get(name).map(|h| h.len()).unwrap_or(0) < 6 {
                        if state.deck.is_empty() {
                            self.check_deck(&mut state);
                        }

                        let card = match state.deck.pop() {
                            Some(card) => card,
                            None => break,
                        };

                        if let Some(hand) = state.player_hand.get_mut(name) {
                            hand.push(card);
                        }
                    }

                    if !state.player_order.iter().any(|player| player == name) {
                        state.player_order.push(name.to_string());
                    }
                }
                RoomStage::Voting | RoomStage::Results => {
                    state.observers.insert(
                        name.to_string(),
                        ObserverInfo {
                            connected: true,
                            points: 0,
                            join_requested: false,
                            auto_join_on_next_round: true,
                        },
                    );
                }
                RoomStage::End => {
                    socket
                        .send(ServerMsg::ErrorMsg("Game has already ended".to_string()).into())
                        .await?;
                    return Err(anyhow!("Game has already ended"));
                }
            }
        }

        state
            .name_tokens
            .insert(name.to_string(), token.to_string());
        let generation = self.next_connection_generation(&mut state);
        state
            .connection_generation
            .insert(name.to_string(), generation);

        if state.creator.as_deref() == Some(name) {
            state.moderators.insert(name.to_string());
        }
        self.clean_moderators(&mut state);
        self.maybe_promote_moderator(&mut state);

        if !is_known_member && matches!(state.stage, RoomStage::Paused) {
            state.paused_reason = Some(self.pause_reason());
        }

        self.broadcast_msg(self.room_state(&state))?; // will not receive this one yet
        socket.send(self.room_state(&state).into()).await?;
        if state.players.contains_key(name) {
            if let Ok(msg) = self.get_msg(Some(name), &state) {
                socket.send(msg.into()).await?;
            }
        } else if matches!(state.stage, RoomStage::Voting | RoomStage::Results) {
            if let Ok(msg) = self.get_msg(Some(name), &state) {
                socket.send(msg.into()).await?;
            }
        }

        Ok(generation)
    }

    async fn run_ws_loop(
        &self,
        socket: &mut WebSocket,
        name: &str,
        connection_generation: u64,
    ) -> Result<()> {
        let (tx, mut rx) = mpsc::channel(10);
        {
            let mut state = self.state.write().await;
            if state.connection_generation.get(name).copied() != Some(connection_generation) {
                return Ok(());
            }
            state.player_to_socket.insert(name.to_string(), tx);
        }
        let mut broadcast_updates = self.broadcast.subscribe();

        loop {
            tokio::select! {
                msg = broadcast_updates.recv() => {
                    if self.state.read().await.connection_generation.get(name).copied() != Some(connection_generation) {
                        break;
                    }
                    socket.send(msg?.into()).await?;
                }
                msg = socket.recv() => {
                    if self.state.read().await.connection_generation.get(name).copied() != Some(connection_generation) {
                        break;
                    }
                    match msg {
                        Some(Ok(WsMessage::Close(_))) => break,
                        Some(Ok(WsMessage::Ping(payload))) => {
                            socket.send(WsMessage::Pong(payload)).await?;
                        }
                        Some(Ok(WsMessage::Pong(_))) => {}
                        Some(Ok(msg)) => {
                            self.handle_client_msg(name, connection_generation, msg).await?;
                        }
                        _ => break
                    }
                },
                msg = rx.recv() => {
                    if self.state.read().await.connection_generation.get(name).copied() != Some(connection_generation) {
                        break;
                    }
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
            observers: state.observers.clone(),
            creator: state.creator.clone(),
            moderators,
            stage: state.stage,
            active_player: state.player_order.get(state.active_player).cloned(),
            player_order: state.player_order.clone(),
            round: state.round,
            cards_remaining: state.deck.len() as u32,
            deck_refill_count: state.deck_refill_count,
            win_condition: state.win_condition,
            allow_new_players_midgame: state.allow_new_players_midgame,
            paused_reason: state.paused_reason.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_room() -> Room {
        let deck = (0..512).map(|i| format!("card-{}", i)).collect::<Vec<_>>();
        Room::new(
            "test",
            Arc::new(deck),
            WinCondition::Points { target_points: 10 },
            Some("host".to_string()),
            64,
        )
    }

    fn add_player(state: &mut RwLockWriteGuard<'_, RoomState>, name: &str, points: u16) {
        state.players.insert(
            name.to_string(),
            PlayerInfo {
                connected: true,
                points,
                ready: false,
            },
        );
        state.player_hand.insert(
            name.to_string(),
            vec![
                format!("{}-1", name),
                format!("{}-2", name),
                format!("{}-3", name),
                format!("{}-4", name),
                format!("{}-5", name),
                format!("{}-6", name),
            ],
        );
        if !state.player_order.iter().any(|player| player == name) {
            state.player_order.push(name.to_string());
        }
    }

    fn setup_connected_member(
        state: &mut RwLockWriteGuard<'_, RoomState>,
        name: &str,
        token: &str,
        generation: u64,
    ) {
        state
            .name_tokens
            .insert(name.to_string(), token.to_string());
        state
            .connection_generation
            .insert(name.to_string(), generation);
    }

    fn to_ws(msg: ClientMsg) -> WsMessage {
        WsMessage::Text(
            serde_json::to_string(&msg).expect("Client message serialization must work"),
        )
    }

    #[tokio::test]
    async fn kicking_non_voter_during_voting_transitions_to_results() -> Result<()> {
        let room = test_room();
        let mut state = room.state.write().await;

        add_player(&mut state, "a", 0);
        add_player(&mut state, "b", 0);
        add_player(&mut state, "c", 0);
        add_player(&mut state, "d", 0);
        state.player_order = vec!["a".into(), "b".into(), "c".into(), "d".into()];
        state.active_player = 0;
        state.round = 2;
        state.stage = RoomStage::Voting;
        state.current_description = "clue".to_string();

        state.player_to_current_card.insert("a".into(), "ca".into());
        state.player_to_current_card.insert("b".into(), "cb".into());
        state.player_to_current_card.insert("c".into(), "cc".into());
        state.player_to_current_card.insert("d".into(), "cd".into());
        state.player_to_vote.insert("b".into(), "ca".into());
        state.player_to_vote.insert("c".into(), "ca".into());
        // d intentionally has not voted yet

        let removed = room.remove_player(&mut state, "d", None).await?;
        assert!(removed, "expected player d to be removed");

        room.after_member_removed_or_observered(&mut state).await?;
        assert!(
            matches!(state.stage, RoomStage::Results),
            "expected to transition to Results after removing non-voter in Voting"
        );

        Ok(())
    }

    #[tokio::test]
    async fn dropping_below_three_players_enters_pause_instead_of_ending() -> Result<()> {
        let room = test_room();
        let mut state = room.state.write().await;

        add_player(&mut state, "a", 0);
        add_player(&mut state, "b", 0);
        add_player(&mut state, "c", 0);
        state.player_order = vec!["a".into(), "b".into(), "c".into()];
        state.active_player = 0;
        state.round = 3;
        state.stage = RoomStage::PlayersChoose;
        state.current_description = "clue".to_string();

        let removed = room.remove_player(&mut state, "c", None).await?;
        assert!(removed, "expected player c to be removed");
        room.after_member_removed_or_observered(&mut state).await?;

        assert!(
            matches!(state.stage, RoomStage::Paused),
            "expected stage to be Paused when active players drop below 3"
        );
        assert!(
            state.paused_reason.is_some(),
            "expected a pause reason to be set"
        );

        Ok(())
    }

    #[tokio::test]
    async fn resume_game_requires_three_non_observer_players() -> Result<()> {
        let room = test_room();

        {
            let mut state = room.state.write().await;
            add_player(&mut state, "host", 2);
            add_player(&mut state, "p2", 3);
            state.moderators.insert("host".to_string());
            state.stage = RoomStage::Paused;
            state.paused_reason = Some("Need at least 3 non-observer players".to_string());
            setup_connected_member(&mut state, "host", "t-host", 7);
        }

        room.handle_client_msg("host", 7, to_ws(ClientMsg::ResumeGame {}))
            .await?;
        {
            let state = room.state.read().await;
            assert!(
                matches!(state.stage, RoomStage::Paused),
                "resume should be rejected while fewer than 3 non-observer players exist"
            );
        }

        {
            let mut state = room.state.write().await;
            add_player(&mut state, "p3", 4);
        }

        room.handle_client_msg("host", 7, to_ws(ClientMsg::ResumeGame {}))
            .await?;
        {
            let state = room.state.read().await;
            assert!(
                matches!(state.stage, RoomStage::ActiveChooses),
                "resume should start a round once at least 3 non-observer players exist"
            );
        }

        Ok(())
    }

    #[tokio::test]
    async fn observer_promotion_uses_floor_score_only() -> Result<()> {
        let room = test_room();
        let mut state = room.state.write().await;

        add_player(&mut state, "p1", 6);
        add_player(&mut state, "p2", 10);

        state.observers.insert(
            "low".to_string(),
            ObserverInfo {
                connected: true,
                points: 2,
                join_requested: true,
                auto_join_on_next_round: false,
            },
        );
        state.observers.insert(
            "high".to_string(),
            ObserverInfo {
                connected: true,
                points: 14,
                join_requested: true,
                auto_join_on_next_round: false,
            },
        );

        let promoted = room.promote_requested_observers(&mut state);
        assert_eq!(promoted, 2, "expected both observers to be promoted");
        assert_eq!(
            state.players.get("low").map(|p| p.points),
            Some(6),
            "low observer should be floored to minimum player score"
        );
        assert_eq!(
            state.players.get("high").map(|p| p.points),
            Some(14),
            "higher observer score should be preserved"
        );

        Ok(())
    }

    #[tokio::test]
    async fn token_validation_enforces_name_ownership() -> Result<()> {
        let room = test_room();
        let mut state = room.state.write().await;
        state
            .name_tokens
            .insert("alice".to_string(), "token-a".to_string());

        assert!(
            room.has_valid_token_for_name(&state, "alice", "token-a"),
            "same token must be accepted"
        );
        assert!(
            !room.has_valid_token_for_name(&state, "alice", "token-b"),
            "different token for same name must be rejected"
        );
        assert!(
            room.has_valid_token_for_name(&state, "bob", "token-any"),
            "new name should accept any token"
        );

        Ok(())
    }
}
