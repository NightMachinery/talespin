export interface PlayerInfo {
	connected: boolean;
	points: number;
	ready: boolean;
}

export interface ObserverInfo {
	connected: boolean;
	points: number | null;
	join_requested: boolean;
	auto_join_on_next_round: boolean;
}

export type GameMode = 'dixit_plus' | 'stella';

export type StellaQueuedRevealMode = 'animated' | 'fast';

export type BeautyResultsDisplayMode = 'summary' | 'separate' | 'combined';

export type BeautyScoringMode = 'vote_divisor' | 'winner_bonus';

export type BeautyVotePointsDivisorMode = 'manual' | 'player_count_auto' | 'median_auto';

export type LeaderboardViewMode =
	| 'total'
	| 'story_only'
	| 'beauty_only'
	| 'combined'
	| 'clue_stars';

export type PreviousDixitResultsView =
	| {
			kind: 'results';
			center_cards: string[];
			player_to_votes: Record<string, string[]>;
			player_to_beauty_votes: Record<string, string[]>;
			player_to_clue_rating: Record<string, number>;
			player_to_current_cards: Record<string, string[]>;
			active_card: string;
			beauty_results_display_mode: BeautyResultsDisplayMode;
			point_change: Record<string, number>;
			storyteller_point_change: Record<string, number>;
			beauty_point_change: Record<string, number>;
			beauty_vote_totals: Record<string, number>;
			beauty_winning_cards: string[];
	  }
	| {
			kind: 'beauty_results';
			center_cards: string[];
			player_to_beauty_votes: Record<string, string[]>;
			player_to_clue_rating: Record<string, number>;
			player_to_current_cards: Record<string, string[]>;
			point_change: Record<string, number>;
			beauty_vote_totals: Record<string, number>;
			beauty_winning_cards: string[];
	  };

export interface DixitEndRoundHistoryEntry {
	round_num: number;
	storyteller: string;
	clue: string;
	active_players: string[];
	story_deltas: Record<string, number>;
	beauty_deltas: Record<string, number>;
	clue_rating_sum: number;
	clue_rating_count: number;
	clue_rating_bonus: number;
	player_clue_ratings: Record<string, number>;
	total_after_round: Record<string, number>;
	beauty_total_after_round: Record<string, number>;
	results_display_mode: BeautyResultsDisplayMode;
}

export type WinCondition =
	| {
			mode: 'points';
			target_points: number;
	  }
	| {
			mode: 'cycles';
			target_cycles: number;
	  }
	| {
			mode: 'fixed_rounds';
			target_rounds: number;
	  }
	| {
			mode: 'cards_finish';
	  };
