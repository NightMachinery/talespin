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
