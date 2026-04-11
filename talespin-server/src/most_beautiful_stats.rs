use anyhow::{Context, Result};
use rusqlite::{params, Connection, OptionalExtension};
use serde::Serialize;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone)]
pub struct MostBeautifulVoteRecord {
    pub voter_hash: String,
    pub voter_display_name: String,
    pub owner_hash: String,
    pub owner_display_name: String,
    pub card_hash: String,
    pub vote_count: u16,
}

#[derive(Debug, Clone)]
pub struct MostBeautifulRoundWinRecord {
    pub winner_hash: String,
    pub winner_display_name: String,
    pub is_tie: bool,
}

#[derive(Debug, Clone)]
pub struct MostBeautifulRoundRecord {
    pub recorded_at_s: u64,
    pub room_id: String,
    pub round_num: u16,
    pub votes: Vec<MostBeautifulVoteRecord>,
    pub wins: Vec<MostBeautifulRoundWinRecord>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct MostBeautifulVoterStats {
    pub player_hash: String,
    pub display_name: String,
    pub votes: u64,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct MostBeautifulPlayerStats {
    pub player_hash: String,
    pub display_name: String,
    pub votes_received: u64,
    pub rounds_won: u64,
    pub tie_round_wins: u64,
    pub decisive_round_wins: u64,
    pub voters: Vec<MostBeautifulVoterStats>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct MostBeautifulStatsResponse {
    pub players: Vec<MostBeautifulPlayerStats>,
}

#[derive(Debug, Clone)]
pub struct MostBeautifulGameAuditCardRecord {
    pub card_hash: String,
    pub owner_hash: String,
    pub owner_display_name: String,
    pub submitted_by_hash: String,
    pub submitted_by_display_name: String,
    pub is_storyteller_card: bool,
    pub center_order: u16,
}

#[derive(Debug, Clone)]
pub struct MostBeautifulGameAuditVoteRecord {
    pub voter_hash: String,
    pub voter_display_name: String,
    pub card_hash: String,
    pub vote_count: u16,
}

#[derive(Debug, Clone)]
pub struct MostBeautifulGameAuditScoreRecord {
    pub player_hash: String,
    pub player_display_name: String,
    pub story_delta: u16,
    pub beauty_delta: u16,
    pub total_after_round: u16,
    pub beauty_total_after_round: u16,
}

#[derive(Debug, Clone)]
pub struct MostBeautifulGameAuditRoundRecord {
    pub game_id: String,
    pub room_id: String,
    pub game_started_at_s: u64,
    pub recorded_at_s: u64,
    pub round_num: u16,
    pub storyteller_hash: String,
    pub storyteller_display_name: String,
    pub clue: String,
    pub results_display_mode: String,
    pub card_entries: Vec<MostBeautifulGameAuditCardRecord>,
    pub story_votes: Vec<MostBeautifulGameAuditVoteRecord>,
    pub beauty_votes: Vec<MostBeautifulGameAuditVoteRecord>,
    pub score_log: Vec<MostBeautifulGameAuditScoreRecord>,
}

#[derive(Debug, Clone)]
pub struct MostBeautifulStatsStore {
    db_path: PathBuf,
}

impl MostBeautifulStatsStore {
    pub fn new(db_path: impl Into<PathBuf>) -> Result<Self> {
        let db_path = db_path.into();
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent).with_context(|| {
                format!(
                    "Failed to create Most Beautiful stats directory {}",
                    parent.display()
                )
            })?;
        }

        let store = Self { db_path };
        store.init()?;
        Ok(store)
    }

    pub fn path(&self) -> &Path {
        &self.db_path
    }

    fn connect(&self) -> Result<Connection> {
        let conn = Connection::open(&self.db_path).with_context(|| {
            format!(
                "Failed to open Most Beautiful stats database {}",
                self.db_path.display()
            )
        })?;
        conn.pragma_update(None, "journal_mode", "WAL")
            .context("Failed to enable WAL mode for Most Beautiful stats")?;
        conn.pragma_update(None, "foreign_keys", "ON")
            .context("Failed to enable foreign keys for Most Beautiful stats")?;
        conn.busy_timeout(std::time::Duration::from_secs(5))
            .context("Failed to configure sqlite busy timeout")?;
        Ok(conn)
    }

    fn init(&self) -> Result<()> {
        let conn = self.connect()?;
        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS mb_players (
                player_hash TEXT PRIMARY KEY,
                latest_display_name TEXT NOT NULL,
                first_seen_at INTEGER NOT NULL,
                last_seen_at INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS mb_player_names (
                player_hash TEXT NOT NULL,
                display_name TEXT NOT NULL,
                first_seen_at INTEGER NOT NULL,
                last_seen_at INTEGER NOT NULL,
                PRIMARY KEY (player_hash, display_name),
                FOREIGN KEY (player_hash) REFERENCES mb_players(player_hash) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS mb_card_paths (
                card_hash TEXT NOT NULL,
                card_path TEXT NOT NULL,
                first_seen_at INTEGER NOT NULL,
                last_seen_at INTEGER NOT NULL,
                PRIMARY KEY (card_hash, card_path)
            );

            CREATE TABLE IF NOT EXISTS mb_vote_events (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                recorded_at INTEGER NOT NULL,
                room_id TEXT NOT NULL,
                round_num INTEGER NOT NULL,
                voter_hash TEXT NOT NULL,
                voter_display_name TEXT NOT NULL,
                owner_hash TEXT NOT NULL,
                owner_display_name TEXT NOT NULL,
                card_hash TEXT NOT NULL,
                vote_count INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS mb_round_win_events (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                recorded_at INTEGER NOT NULL,
                room_id TEXT NOT NULL,
                round_num INTEGER NOT NULL,
                winner_hash TEXT NOT NULL,
                winner_display_name TEXT NOT NULL,
                is_tie INTEGER NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_mb_vote_owner ON mb_vote_events(owner_hash);
            CREATE INDEX IF NOT EXISTS idx_mb_vote_voter ON mb_vote_events(voter_hash);
            CREATE INDEX IF NOT EXISTS idx_mb_vote_card ON mb_vote_events(card_hash);
            CREATE INDEX IF NOT EXISTS idx_mb_round_winner ON mb_round_win_events(winner_hash);

            CREATE TABLE IF NOT EXISTS mb_games (
                game_id TEXT PRIMARY KEY,
                room_id TEXT NOT NULL,
                started_at INTEGER NOT NULL,
                ended_at INTEGER,
                completed INTEGER NOT NULL DEFAULT 0,
                total_rounds INTEGER NOT NULL DEFAULT 0
            );

            CREATE TABLE IF NOT EXISTS mb_game_rounds (
                game_id TEXT NOT NULL,
                round_num INTEGER NOT NULL,
                recorded_at INTEGER NOT NULL,
                storyteller_hash TEXT NOT NULL,
                storyteller_display_name TEXT NOT NULL,
                clue TEXT NOT NULL,
                results_display_mode TEXT NOT NULL,
                PRIMARY KEY (game_id, round_num),
                FOREIGN KEY (game_id) REFERENCES mb_games(game_id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS mb_game_round_cards (
                game_id TEXT NOT NULL,
                round_num INTEGER NOT NULL,
                card_hash TEXT NOT NULL,
                owner_hash TEXT NOT NULL,
                owner_display_name TEXT NOT NULL,
                submitted_by_hash TEXT NOT NULL,
                submitted_by_display_name TEXT NOT NULL,
                is_storyteller_card INTEGER NOT NULL,
                center_order INTEGER NOT NULL,
                PRIMARY KEY (game_id, round_num, card_hash),
                FOREIGN KEY (game_id, round_num)
                    REFERENCES mb_game_rounds(game_id, round_num)
                    ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS mb_game_round_story_votes (
                game_id TEXT NOT NULL,
                round_num INTEGER NOT NULL,
                voter_hash TEXT NOT NULL,
                voter_display_name TEXT NOT NULL,
                card_hash TEXT NOT NULL,
                vote_count INTEGER NOT NULL,
                PRIMARY KEY (game_id, round_num, voter_hash, card_hash),
                FOREIGN KEY (game_id, round_num)
                    REFERENCES mb_game_rounds(game_id, round_num)
                    ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS mb_game_round_beauty_votes (
                game_id TEXT NOT NULL,
                round_num INTEGER NOT NULL,
                voter_hash TEXT NOT NULL,
                voter_display_name TEXT NOT NULL,
                card_hash TEXT NOT NULL,
                vote_count INTEGER NOT NULL,
                PRIMARY KEY (game_id, round_num, voter_hash, card_hash),
                FOREIGN KEY (game_id, round_num)
                    REFERENCES mb_game_rounds(game_id, round_num)
                    ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS mb_game_round_scores (
                game_id TEXT NOT NULL,
                round_num INTEGER NOT NULL,
                player_hash TEXT NOT NULL,
                player_display_name TEXT NOT NULL,
                story_delta INTEGER NOT NULL,
                beauty_delta INTEGER NOT NULL,
                total_after_round INTEGER NOT NULL,
                beauty_total_after_round INTEGER NOT NULL,
                PRIMARY KEY (game_id, round_num, player_hash),
                FOREIGN KEY (game_id, round_num)
                    REFERENCES mb_game_rounds(game_id, round_num)
                    ON DELETE CASCADE
            );

            CREATE INDEX IF NOT EXISTS idx_mb_game_round_storyteller
                ON mb_game_rounds(storyteller_hash);
            CREATE INDEX IF NOT EXISTS idx_mb_game_round_card_owner
                ON mb_game_round_cards(owner_hash);
            CREATE INDEX IF NOT EXISTS idx_mb_game_story_vote_voter
                ON mb_game_round_story_votes(voter_hash);
            CREATE INDEX IF NOT EXISTS idx_mb_game_beauty_vote_voter
                ON mb_game_round_beauty_votes(voter_hash);
            CREATE INDEX IF NOT EXISTS idx_mb_game_round_score_player
                ON mb_game_round_scores(player_hash);
            "#,
        )
        .context("Failed to initialize Most Beautiful stats schema")?;
        Ok(())
    }

    fn upsert_player(
        tx: &rusqlite::Transaction<'_>,
        player_hash: &str,
        display_name: &str,
        recorded_at_s: u64,
    ) -> Result<()> {
        tx.execute(
            r#"
            INSERT INTO mb_players (player_hash, latest_display_name, first_seen_at, last_seen_at)
            VALUES (?1, ?2, ?3, ?3)
            ON CONFLICT(player_hash) DO UPDATE SET
                latest_display_name = excluded.latest_display_name,
                last_seen_at = excluded.last_seen_at
            "#,
            params![player_hash, display_name, recorded_at_s],
        )
        .context("Failed to upsert Most Beautiful player")?;

        tx.execute(
            r#"
            INSERT INTO mb_player_names (player_hash, display_name, first_seen_at, last_seen_at)
            VALUES (?1, ?2, ?3, ?3)
            ON CONFLICT(player_hash, display_name) DO UPDATE SET
                last_seen_at = excluded.last_seen_at
            "#,
            params![player_hash, display_name, recorded_at_s],
        )
        .context("Failed to upsert Most Beautiful player alias")?;

        Ok(())
    }

    pub fn register_card_paths(
        &self,
        recorded_at_s: u64,
        card_paths: &HashMap<String, PathBuf>,
    ) -> Result<()> {
        let mut conn = self.connect()?;
        let tx = conn
            .transaction()
            .context("Failed to open Most Beautiful card-path transaction")?;
        for (card_hash, path) in card_paths {
            let path_str = path.to_string_lossy().to_string();
            tx.execute(
                r#"
                INSERT INTO mb_card_paths (card_hash, card_path, first_seen_at, last_seen_at)
                VALUES (?1, ?2, ?3, ?3)
                ON CONFLICT(card_hash, card_path) DO UPDATE SET
                    last_seen_at = excluded.last_seen_at
                "#,
                params![card_hash, path_str, recorded_at_s],
            )
            .context("Failed to upsert Most Beautiful card path")?;
        }
        tx.commit()
            .context("Failed to commit Most Beautiful card-path transaction")?;
        Ok(())
    }

    pub fn record_round(&self, record: &MostBeautifulRoundRecord) -> Result<()> {
        if record.votes.is_empty() && record.wins.is_empty() {
            return Ok(());
        }

        let mut conn = self.connect()?;
        let tx = conn
            .transaction()
            .context("Failed to open Most Beautiful stats transaction")?;

        for vote in &record.votes {
            Self::upsert_player(
                &tx,
                &vote.voter_hash,
                &vote.voter_display_name,
                record.recorded_at_s,
            )?;
            Self::upsert_player(
                &tx,
                &vote.owner_hash,
                &vote.owner_display_name,
                record.recorded_at_s,
            )?;
            tx.execute(
                r#"
                INSERT INTO mb_vote_events (
                    recorded_at,
                    room_id,
                    round_num,
                    voter_hash,
                    voter_display_name,
                    owner_hash,
                    owner_display_name,
                    card_hash,
                    vote_count
                )
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
                "#,
                params![
                    record.recorded_at_s,
                    record.room_id,
                    record.round_num,
                    vote.voter_hash,
                    vote.voter_display_name,
                    vote.owner_hash,
                    vote.owner_display_name,
                    vote.card_hash,
                    vote.vote_count
                ],
            )
            .context("Failed to insert Most Beautiful vote event")?;
        }

        for win in &record.wins {
            Self::upsert_player(
                &tx,
                &win.winner_hash,
                &win.winner_display_name,
                record.recorded_at_s,
            )?;
            tx.execute(
                r#"
                INSERT INTO mb_round_win_events (
                    recorded_at,
                    room_id,
                    round_num,
                    winner_hash,
                    winner_display_name,
                    is_tie
                )
                VALUES (?1, ?2, ?3, ?4, ?5, ?6)
                "#,
                params![
                    record.recorded_at_s,
                    record.room_id,
                    record.round_num,
                    win.winner_hash,
                    win.winner_display_name,
                    i64::from(win.is_tie)
                ],
            )
            .context("Failed to insert Most Beautiful round-win event")?;
        }

        tx.commit()
            .context("Failed to commit Most Beautiful stats transaction")?;
        Ok(())
    }

    pub fn record_game_audit_round(
        &self,
        record: &MostBeautifulGameAuditRoundRecord,
    ) -> Result<()> {
        let mut conn = self.connect()?;
        let tx = conn
            .transaction()
            .context("Failed to open Most Beautiful game-audit transaction")?;

        tx.execute(
            r#"
            INSERT INTO mb_games (game_id, room_id, started_at, total_rounds)
            VALUES (?1, ?2, ?3, ?4)
            ON CONFLICT(game_id) DO UPDATE SET
                room_id = excluded.room_id,
                started_at = MIN(started_at, excluded.started_at),
                total_rounds = MAX(total_rounds, excluded.total_rounds)
            "#,
            params![
                record.game_id,
                record.room_id,
                record.game_started_at_s,
                record.round_num
            ],
        )
        .context("Failed to upsert Most Beautiful game audit row")?;

        Self::upsert_player(
            &tx,
            &record.storyteller_hash,
            &record.storyteller_display_name,
            record.recorded_at_s,
        )?;

        tx.execute(
            r#"
            INSERT INTO mb_game_rounds (
                game_id,
                round_num,
                recorded_at,
                storyteller_hash,
                storyteller_display_name,
                clue,
                results_display_mode
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            ON CONFLICT(game_id, round_num) DO UPDATE SET
                recorded_at = excluded.recorded_at,
                storyteller_hash = excluded.storyteller_hash,
                storyteller_display_name = excluded.storyteller_display_name,
                clue = excluded.clue,
                results_display_mode = excluded.results_display_mode
            "#,
            params![
                record.game_id,
                record.round_num,
                record.recorded_at_s,
                record.storyteller_hash,
                record.storyteller_display_name,
                record.clue,
                record.results_display_mode
            ],
        )
        .context("Failed to upsert Most Beautiful game round row")?;

        tx.execute(
            "DELETE FROM mb_game_round_cards WHERE game_id = ?1 AND round_num = ?2",
            params![record.game_id, record.round_num],
        )
        .context("Failed to clear prior Most Beautiful game round cards")?;
        tx.execute(
            "DELETE FROM mb_game_round_story_votes WHERE game_id = ?1 AND round_num = ?2",
            params![record.game_id, record.round_num],
        )
        .context("Failed to clear prior Most Beautiful game round storyteller votes")?;
        tx.execute(
            "DELETE FROM mb_game_round_beauty_votes WHERE game_id = ?1 AND round_num = ?2",
            params![record.game_id, record.round_num],
        )
        .context("Failed to clear prior Most Beautiful game round beauty votes")?;
        tx.execute(
            "DELETE FROM mb_game_round_scores WHERE game_id = ?1 AND round_num = ?2",
            params![record.game_id, record.round_num],
        )
        .context("Failed to clear prior Most Beautiful game round scores")?;

        for card in &record.card_entries {
            Self::upsert_player(
                &tx,
                &card.owner_hash,
                &card.owner_display_name,
                record.recorded_at_s,
            )?;
            Self::upsert_player(
                &tx,
                &card.submitted_by_hash,
                &card.submitted_by_display_name,
                record.recorded_at_s,
            )?;
            tx.execute(
                r#"
                INSERT INTO mb_game_round_cards (
                    game_id,
                    round_num,
                    card_hash,
                    owner_hash,
                    owner_display_name,
                    submitted_by_hash,
                    submitted_by_display_name,
                    is_storyteller_card,
                    center_order
                )
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
                "#,
                params![
                    record.game_id,
                    record.round_num,
                    card.card_hash,
                    card.owner_hash,
                    card.owner_display_name,
                    card.submitted_by_hash,
                    card.submitted_by_display_name,
                    i64::from(card.is_storyteller_card),
                    card.center_order
                ],
            )
            .context("Failed to insert Most Beautiful game round card")?;
        }

        Self::insert_game_vote_records(
            &tx,
            "mb_game_round_story_votes",
            record,
            &record.story_votes,
        )?;
        Self::insert_game_vote_records(
            &tx,
            "mb_game_round_beauty_votes",
            record,
            &record.beauty_votes,
        )?;

        for score in &record.score_log {
            Self::upsert_player(
                &tx,
                &score.player_hash,
                &score.player_display_name,
                record.recorded_at_s,
            )?;
            tx.execute(
                r#"
                INSERT INTO mb_game_round_scores (
                    game_id,
                    round_num,
                    player_hash,
                    player_display_name,
                    story_delta,
                    beauty_delta,
                    total_after_round,
                    beauty_total_after_round
                )
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
                "#,
                params![
                    record.game_id,
                    record.round_num,
                    score.player_hash,
                    score.player_display_name,
                    score.story_delta,
                    score.beauty_delta,
                    score.total_after_round,
                    score.beauty_total_after_round
                ],
            )
            .context("Failed to insert Most Beautiful game round score")?;
        }

        tx.commit()
            .context("Failed to commit Most Beautiful game-audit transaction")?;
        Ok(())
    }

    fn insert_game_vote_records(
        tx: &rusqlite::Transaction<'_>,
        table_name: &str,
        record: &MostBeautifulGameAuditRoundRecord,
        votes: &[MostBeautifulGameAuditVoteRecord],
    ) -> Result<()> {
        let query = format!(
            "INSERT INTO {} (game_id, round_num, voter_hash, voter_display_name, card_hash, vote_count) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            table_name
        );
        for vote in votes {
            Self::upsert_player(
                tx,
                &vote.voter_hash,
                &vote.voter_display_name,
                record.recorded_at_s,
            )?;
            tx.execute(
                &query,
                params![
                    record.game_id,
                    record.round_num,
                    vote.voter_hash,
                    vote.voter_display_name,
                    vote.card_hash,
                    vote.vote_count
                ],
            )
            .with_context(|| {
                format!(
                    "Failed to insert Most Beautiful game vote into {}",
                    table_name
                )
            })?;
        }
        Ok(())
    }

    pub fn mark_game_complete(
        &self,
        game_id: &str,
        ended_at_s: u64,
        total_rounds: u16,
    ) -> Result<()> {
        let conn = self.connect()?;
        conn.execute(
            r#"
            UPDATE mb_games
            SET ended_at = ?2,
                completed = 1,
                total_rounds = MAX(total_rounds, ?3)
            WHERE game_id = ?1
            "#,
            params![game_id, ended_at_s, total_rounds],
        )
        .context("Failed to mark Most Beautiful game audit complete")?;
        Ok(())
    }

    pub fn aggregated_stats(&self) -> Result<MostBeautifulStatsResponse> {
        let conn = self.connect()?;
        let mut players = Self::load_players(&conn)?;
        Self::load_vote_totals(&conn, &mut players)?;
        Self::load_round_wins(&conn, &mut players)?;

        let mut player_entries = players.into_values().collect::<Vec<_>>();
        player_entries.sort_by(|a, b| {
            b.rounds_won
                .cmp(&a.rounds_won)
                .then_with(|| b.votes_received.cmp(&a.votes_received))
                .then_with(|| {
                    a.display_name
                        .to_lowercase()
                        .cmp(&b.display_name.to_lowercase())
                })
                .then_with(|| a.player_hash.cmp(&b.player_hash))
        });
        for player in &mut player_entries {
            player.voters.sort_by(|a, b| {
                b.votes
                    .cmp(&a.votes)
                    .then_with(|| {
                        a.display_name
                            .to_lowercase()
                            .cmp(&b.display_name.to_lowercase())
                    })
                    .then_with(|| a.player_hash.cmp(&b.player_hash))
            });
        }

        Ok(MostBeautifulStatsResponse {
            players: player_entries,
        })
    }

    fn load_players(conn: &Connection) -> Result<HashMap<String, MostBeautifulPlayerStats>> {
        let mut stmt = conn
            .prepare(
                "SELECT player_hash, latest_display_name FROM mb_players ORDER BY latest_display_name, player_hash",
            )
            .context("Failed to prepare Most Beautiful players query")?;
        let rows = stmt
            .query_map([], |row| {
                Ok(MostBeautifulPlayerStats {
                    player_hash: row.get(0)?,
                    display_name: row.get(1)?,
                    votes_received: 0,
                    rounds_won: 0,
                    tie_round_wins: 0,
                    decisive_round_wins: 0,
                    voters: Vec::new(),
                })
            })
            .context("Failed to query Most Beautiful players")?;

        let mut players = HashMap::new();
        for row in rows {
            let player = row.context("Failed to decode Most Beautiful player row")?;
            players.insert(player.player_hash.clone(), player);
        }
        Ok(players)
    }

    fn player_name(conn: &Connection, player_hash: &str, fallback: &str) -> Result<String> {
        conn.query_row(
            "SELECT latest_display_name FROM mb_players WHERE player_hash = ?1",
            params![player_hash],
            |row| row.get(0),
        )
        .optional()
        .map(|name| name.unwrap_or_else(|| fallback.to_string()))
        .context("Failed to load Most Beautiful player display name")
    }

    fn load_vote_totals(
        conn: &Connection,
        players: &mut HashMap<String, MostBeautifulPlayerStats>,
    ) -> Result<()> {
        let mut totals_stmt = conn
            .prepare(
                r#"
                SELECT owner_hash, COALESCE(SUM(vote_count), 0)
                FROM mb_vote_events
                GROUP BY owner_hash
                "#,
            )
            .context("Failed to prepare Most Beautiful vote totals query")?;
        let vote_rows = totals_stmt
            .query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, u64>(1)?))
            })
            .context("Failed to query Most Beautiful vote totals")?;
        for row in vote_rows {
            let (owner_hash, total_votes) =
                row.context("Failed to decode Most Beautiful votes row")?;
            players
                .entry(owner_hash.clone())
                .or_insert(MostBeautifulPlayerStats {
                    player_hash: owner_hash.clone(),
                    display_name: Self::player_name(conn, &owner_hash, &owner_hash)?,
                    votes_received: 0,
                    rounds_won: 0,
                    tie_round_wins: 0,
                    decisive_round_wins: 0,
                    voters: Vec::new(),
                })
                .votes_received = total_votes;
        }

        let mut breakdown_stmt = conn
            .prepare(
                r#"
                SELECT owner_hash, voter_hash, SUM(vote_count), MAX(voter_display_name)
                FROM mb_vote_events
                GROUP BY owner_hash, voter_hash
                "#,
            )
            .context("Failed to prepare Most Beautiful vote breakdown query")?;
        let breakdown_rows = breakdown_stmt
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, u64>(2)?,
                    row.get::<_, String>(3)?,
                ))
            })
            .context("Failed to query Most Beautiful vote breakdown")?;
        for row in breakdown_rows {
            let (owner_hash, voter_hash, votes, voter_display_name) =
                row.context("Failed to decode Most Beautiful vote breakdown row")?;
            let owner = players
                .entry(owner_hash.clone())
                .or_insert(MostBeautifulPlayerStats {
                    player_hash: owner_hash.clone(),
                    display_name: Self::player_name(conn, &owner_hash, &owner_hash)?,
                    votes_received: 0,
                    rounds_won: 0,
                    tie_round_wins: 0,
                    decisive_round_wins: 0,
                    voters: Vec::new(),
                });
            owner.voters.push(MostBeautifulVoterStats {
                player_hash: voter_hash.clone(),
                display_name: Self::player_name(conn, &voter_hash, &voter_display_name)?,
                votes,
            });
        }

        Ok(())
    }

    fn load_round_wins(
        conn: &Connection,
        players: &mut HashMap<String, MostBeautifulPlayerStats>,
    ) -> Result<()> {
        let mut stmt = conn
            .prepare(
                r#"
                SELECT winner_hash, is_tie, COUNT(*)
                FROM mb_round_win_events
                GROUP BY winner_hash, is_tie
                "#,
            )
            .context("Failed to prepare Most Beautiful round wins query")?;
        let rows = stmt
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, bool>(1)?,
                    row.get::<_, u64>(2)?,
                ))
            })
            .context("Failed to query Most Beautiful round wins")?;
        for row in rows {
            let (winner_hash, is_tie, count) =
                row.context("Failed to decode Most Beautiful round wins row")?;
            let player = players
                .entry(winner_hash.clone())
                .or_insert(MostBeautifulPlayerStats {
                    player_hash: winner_hash.clone(),
                    display_name: Self::player_name(conn, &winner_hash, &winner_hash)?,
                    votes_received: 0,
                    rounds_won: 0,
                    tie_round_wins: 0,
                    decisive_round_wins: 0,
                    voters: Vec::new(),
                });
            player.rounds_won += count;
            if is_tie {
                player.tie_round_wins += count;
            } else {
                player.decisive_round_wins += count;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_db_path() -> PathBuf {
        std::env::temp_dir().join(format!(
            "talespin-most-beautiful-{}-{}.sqlite3",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        ))
    }

    #[test]
    fn store_aggregates_votes_and_round_wins_across_rooms() -> Result<()> {
        let path = temp_db_path();
        let store = MostBeautifulStatsStore::new(&path)?;
        store.register_card_paths(
            1,
            &HashMap::from([
                ("card-a".to_string(), PathBuf::from("/tmp/card-a.avif")),
                ("card-b".to_string(), PathBuf::from("/tmp/card-b.avif")),
            ]),
        )?;
        store.register_card_paths(
            2,
            &HashMap::from([(
                "card-a".to_string(),
                PathBuf::from("/tmp/alternate-card-a.avif"),
            )]),
        )?;

        store.record_round(&MostBeautifulRoundRecord {
            recorded_at_s: 1,
            room_id: "room-a".to_string(),
            round_num: 1,
            votes: vec![
                MostBeautifulVoteRecord {
                    voter_hash: "voter-a".to_string(),
                    voter_display_name: "Voter A".to_string(),
                    owner_hash: "owner-a".to_string(),
                    owner_display_name: "Owner A".to_string(),
                    card_hash: "card-a".to_string(),
                    vote_count: 2,
                },
                MostBeautifulVoteRecord {
                    voter_hash: "voter-b".to_string(),
                    voter_display_name: "Voter B".to_string(),
                    owner_hash: "owner-a".to_string(),
                    owner_display_name: "Owner A".to_string(),
                    card_hash: "card-a".to_string(),
                    vote_count: 1,
                },
            ],
            wins: vec![MostBeautifulRoundWinRecord {
                winner_hash: "owner-a".to_string(),
                winner_display_name: "Owner A".to_string(),
                is_tie: false,
            }],
        })?;

        store.record_round(&MostBeautifulRoundRecord {
            recorded_at_s: 2,
            room_id: "room-b".to_string(),
            round_num: 3,
            votes: vec![MostBeautifulVoteRecord {
                voter_hash: "voter-a".to_string(),
                voter_display_name: "Voter A Renamed".to_string(),
                owner_hash: "owner-b".to_string(),
                owner_display_name: "Owner B".to_string(),
                card_hash: "card-b".to_string(),
                vote_count: 1,
            }],
            wins: vec![
                MostBeautifulRoundWinRecord {
                    winner_hash: "owner-a".to_string(),
                    winner_display_name: "Owner A".to_string(),
                    is_tie: true,
                },
                MostBeautifulRoundWinRecord {
                    winner_hash: "owner-b".to_string(),
                    winner_display_name: "Owner B".to_string(),
                    is_tie: true,
                },
            ],
        })?;

        let stats = store.aggregated_stats()?;
        assert_eq!(
            stats.players.len(),
            4,
            "players and voters should all be retained"
        );

        let owner_a = stats
            .players
            .iter()
            .find(|entry| entry.player_hash == "owner-a")
            .context("missing owner-a stats")?;
        assert_eq!(owner_a.display_name, "Owner A");
        assert_eq!(owner_a.votes_received, 3);
        assert_eq!(owner_a.rounds_won, 2);
        assert_eq!(owner_a.tie_round_wins, 1);
        assert_eq!(owner_a.decisive_round_wins, 1);
        assert_eq!(owner_a.voters.len(), 2);
        assert_eq!(owner_a.voters[0].display_name, "Voter A Renamed");
        assert_eq!(owner_a.voters[0].votes, 2);

        let owner_b = stats
            .players
            .iter()
            .find(|entry| entry.player_hash == "owner-b")
            .context("missing owner-b stats")?;
        assert_eq!(owner_b.votes_received, 1);
        assert_eq!(owner_b.rounds_won, 1);
        assert_eq!(owner_b.tie_round_wins, 1);
        assert_eq!(owner_b.decisive_round_wins, 0);

        let conn = store.connect()?;
        let mut card_path_stmt = conn.prepare(
            "SELECT card_path FROM mb_card_paths WHERE card_hash = ?1 ORDER BY card_path",
        )?;
        let card_paths = card_path_stmt
            .query_map(params!["card-a"], |row| row.get::<_, String>(0))?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        assert_eq!(
            card_paths,
            vec![
                "/tmp/alternate-card-a.avif".to_string(),
                "/tmp/card-a.avif".to_string()
            ]
        );

        let stored_vote_card_hash: Option<String> = conn
            .query_row(
                "SELECT card_hash FROM mb_vote_events WHERE owner_hash = ?1 LIMIT 1",
                params!["owner-a"],
                |row| row.get(0),
            )
            .optional()?;
        assert_eq!(stored_vote_card_hash.as_deref(), Some("card-a"));

        std::fs::remove_file(&path).ok();
        Ok(())
    }

    #[test]
    fn store_persists_game_audit_rounds_and_completion() -> Result<()> {
        let path = temp_db_path();
        let store = MostBeautifulStatsStore::new(&path)?;

        store.record_game_audit_round(&MostBeautifulGameAuditRoundRecord {
            game_id: "game-1".to_string(),
            room_id: "room-a".to_string(),
            game_started_at_s: 10,
            recorded_at_s: 12,
            round_num: 1,
            storyteller_hash: "story-h".to_string(),
            storyteller_display_name: "Story".to_string(),
            clue: "moon".to_string(),
            results_display_mode: "combined".to_string(),
            card_entries: vec![
                MostBeautifulGameAuditCardRecord {
                    card_hash: "card-a".to_string(),
                    owner_hash: "story-h".to_string(),
                    owner_display_name: "Story".to_string(),
                    submitted_by_hash: "story-h".to_string(),
                    submitted_by_display_name: "Story".to_string(),
                    is_storyteller_card: true,
                    center_order: 0,
                },
                MostBeautifulGameAuditCardRecord {
                    card_hash: "card-b".to_string(),
                    owner_hash: "p2-h".to_string(),
                    owner_display_name: "P2".to_string(),
                    submitted_by_hash: "p2-h".to_string(),
                    submitted_by_display_name: "P2".to_string(),
                    is_storyteller_card: false,
                    center_order: 1,
                },
            ],
            story_votes: vec![MostBeautifulGameAuditVoteRecord {
                voter_hash: "p2-h".to_string(),
                voter_display_name: "P2".to_string(),
                card_hash: "card-a".to_string(),
                vote_count: 1,
            }],
            beauty_votes: vec![MostBeautifulGameAuditVoteRecord {
                voter_hash: "story-h".to_string(),
                voter_display_name: "Story".to_string(),
                card_hash: "card-b".to_string(),
                vote_count: 1,
            }],
            score_log: vec![
                MostBeautifulGameAuditScoreRecord {
                    player_hash: "story-h".to_string(),
                    player_display_name: "Story".to_string(),
                    story_delta: 3,
                    beauty_delta: 0,
                    total_after_round: 3,
                    beauty_total_after_round: 0,
                },
                MostBeautifulGameAuditScoreRecord {
                    player_hash: "p2-h".to_string(),
                    player_display_name: "P2".to_string(),
                    story_delta: 3,
                    beauty_delta: 2,
                    total_after_round: 5,
                    beauty_total_after_round: 2,
                },
            ],
        })?;
        store.mark_game_complete("game-1", 20, 1)?;

        let conn = store.connect()?;
        let completed: (bool, u64, u64) = conn.query_row(
            "SELECT completed, ended_at, total_rounds FROM mb_games WHERE game_id = ?1",
            params!["game-1"],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )?;
        assert_eq!(completed, (true, 20, 1));

        let card_count: u64 = conn.query_row(
            "SELECT COUNT(*) FROM mb_game_round_cards WHERE game_id = ?1 AND round_num = 1",
            params!["game-1"],
            |row| row.get(0),
        )?;
        assert_eq!(card_count, 2);

        let beauty_vote_count: u64 = conn.query_row(
            "SELECT COUNT(*) FROM mb_game_round_beauty_votes WHERE game_id = ?1 AND round_num = 1",
            params!["game-1"],
            |row| row.get(0),
        )?;
        assert_eq!(beauty_vote_count, 1);

        let p2_score: (u16, u16) = conn.query_row(
            "SELECT total_after_round, beauty_total_after_round FROM mb_game_round_scores WHERE game_id = ?1 AND round_num = 1 AND player_hash = ?2",
            params!["game-1", "p2-h"],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )?;
        assert_eq!(p2_score, (5, 2));

        std::fs::remove_file(&path).ok();
        Ok(())
    }
}
