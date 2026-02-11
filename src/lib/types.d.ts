export interface PlayerInfo {
	connected: boolean;
	points: number;
	ready: boolean;
}

export interface ObserverInfo {
	connected: boolean;
	points: number;
	join_requested: boolean;
	auto_join_on_next_round: boolean;
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
			mode: 'cards_finish';
	  };
