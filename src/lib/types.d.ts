export interface PlayerInfo {
	connected: boolean;
	points: number;
	ready: boolean;
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
