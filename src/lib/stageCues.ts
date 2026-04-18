export type CueNote = {
	frequencyHz: number;
	durationS: number;
};

export const STAGE_CHANGE_CUE_STAGES = [
	'ActiveChooses',
	'PlayersChoose',
	'Voting',
	'BeautyVoting',
	'ClueRating',
	'Results',
	'BeautyResults',
	'StellaAssociate',
	'StellaReveal',
	'StellaResults',
	'Paused',
	'End'
] as const;

export type StageCueStage = (typeof STAGE_CHANGE_CUE_STAGES)[number];
export type GameCueId = StageCueStage | 'ScoutTurn';
export type CueVisualTone = 'choose' | 'vote' | 'results' | 'paused' | 'end';

export type GameCueDefinition = {
	notes: readonly CueNote[];
	oscillatorType?: OscillatorType;
	volume?: number;
	visualTone?: CueVisualTone;
};

export const DEFAULT_CUE_VOLUME = 0.065;

const STAGE_CHANGE_CUE_STAGE_SET = new Set<string>(STAGE_CHANGE_CUE_STAGES);

export function isStageCueStage(stage: string): stage is StageCueStage {
	return STAGE_CHANGE_CUE_STAGE_SET.has(stage);
}

export const GAME_CUES: Record<GameCueId, GameCueDefinition> = {
	ActiveChooses: {
		notes: [
			{ frequencyHz: 523.25, durationS: 0.18 },
			{ frequencyHz: 659.25, durationS: 0.18 }
		],
		visualTone: 'choose'
	},
	PlayersChoose: {
		notes: [
			{ frequencyHz: 587.33, durationS: 0.18 },
			{ frequencyHz: 698.46, durationS: 0.18 }
		],
		visualTone: 'choose'
	},
	Voting: {
		notes: [
			{ frequencyHz: 783.99, durationS: 0.18 },
			{ frequencyHz: 987.77, durationS: 0.18 }
		],
		visualTone: 'vote'
	},
	BeautyVoting: {
		notes: [
			{ frequencyHz: 698.46, durationS: 0.16 },
			{ frequencyHz: 880, durationS: 0.18 },
			{ frequencyHz: 1046.5, durationS: 0.18 }
		],
		visualTone: 'vote'
	},
	ClueRating: {
		notes: [
			{ frequencyHz: 880, durationS: 0.15 },
			{ frequencyHz: 1174.66, durationS: 0.17 },
			{ frequencyHz: 987.77, durationS: 0.19 }
		],
		visualTone: 'vote'
	},
	Results: {
		notes: [
			{ frequencyHz: 987.77, durationS: 0.16 },
			{ frequencyHz: 783.99, durationS: 0.16 },
			{ frequencyHz: 659.25, durationS: 0.2 }
		],
		oscillatorType: 'triangle',
		visualTone: 'results'
	},
	BeautyResults: {
		notes: [
			{ frequencyHz: 1046.5, durationS: 0.16 },
			{ frequencyHz: 1318.51, durationS: 0.18 },
			{ frequencyHz: 1174.66, durationS: 0.22 }
		],
		visualTone: 'results'
	},
	StellaAssociate: {
		notes: [
			{ frequencyHz: 440, durationS: 0.14 },
			{ frequencyHz: 554.37, durationS: 0.16 },
			{ frequencyHz: 659.25, durationS: 0.18 }
		],
		visualTone: 'choose'
	},
	StellaReveal: {
		notes: [
			{ frequencyHz: 659.25, durationS: 0.15 },
			{ frequencyHz: 783.99, durationS: 0.15 },
			{ frequencyHz: 880, durationS: 0.2 }
		],
		visualTone: 'vote'
	},
	StellaResults: {
		notes: [
			{ frequencyHz: 880, durationS: 0.16 },
			{ frequencyHz: 1174.66, durationS: 0.18 },
			{ frequencyHz: 987.77, durationS: 0.22 }
		],
		oscillatorType: 'triangle',
		visualTone: 'results'
	},
	Paused: {
		notes: [
			{ frequencyHz: 659.25, durationS: 0.16 },
			{ frequencyHz: 523.25, durationS: 0.22 }
		],
		oscillatorType: 'triangle',
		visualTone: 'paused'
	},
	End: {
		notes: [
			{ frequencyHz: 783.99, durationS: 0.16 },
			{ frequencyHz: 1046.5, durationS: 0.18 },
			{ frequencyHz: 1318.51, durationS: 0.24 }
		],
		oscillatorType: 'triangle',
		visualTone: 'end'
	},
	ScoutTurn: {
		notes: [
			{ frequencyHz: 740, durationS: 0.1 },
			{ frequencyHz: 987.77, durationS: 0.16 },
			{ frequencyHz: 1318.51, durationS: 0.2 }
		],
		oscillatorType: 'triangle'
	}
};

export function getCueDefinition(cueId: GameCueId) {
	return GAME_CUES[cueId];
}

export function getCueVisualTone(cueId: GameCueId): CueVisualTone | null {
	return GAME_CUES[cueId].visualTone ?? null;
}
