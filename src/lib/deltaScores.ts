import type { PreviousDixitResultsView } from './types';

export type DeltaScoresStage = 'Results' | 'BeautyResults' | 'StellaReveal' | 'StellaResults';
export type DeltaDisplayMode = 'always' | 'only_in_previous_results';

export interface DeltaScores {
	stage: DeltaScoresStage;
	pointChange: Record<string, number>;
	storyPointChange: Record<string, number>;
	beautyPointChange: Record<string, number>;
}

export interface CurrentRoundDeltaInput {
	stage: string;
	pointChange: Record<string, number>;
	storytellerPointChange: Record<string, number>;
	beautyPointChange: Record<string, number>;
}

export function normalizeDeltaDisplayMode(value: string | undefined): DeltaDisplayMode {
	return value === 'only_in_previous_results' ? 'only_in_previous_results' : 'always';
}

export function talespinDeltaDisplayMode(): DeltaDisplayMode {
	return normalizeDeltaDisplayMode(import.meta.env.TALESPIN_DELTA_DISPLAY_MODE);
}

export function previousRoundDeltaScores(
	snapshot: PreviousDixitResultsView | null
): DeltaScores | null {
	if (!snapshot) return null;

	if (snapshot.kind === 'beauty_results') {
		return {
			stage: 'BeautyResults',
			pointChange: snapshot.point_change ?? {},
			storyPointChange: {},
			beautyPointChange: snapshot.point_change ?? {}
		};
	}

	return {
		stage: 'Results',
		pointChange: snapshot.point_change ?? {},
		storyPointChange: snapshot.storyteller_point_change ?? {},
		beautyPointChange: snapshot.beauty_point_change ?? {}
	};
}

export function currentRoundDeltaScores(input: CurrentRoundDeltaInput): DeltaScores | null {
	switch (input.stage) {
		case 'Results':
			return {
				stage: 'Results',
				pointChange: input.pointChange ?? {},
				storyPointChange: input.storytellerPointChange ?? {},
				beautyPointChange: input.beautyPointChange ?? {}
			};
		case 'BeautyResults':
			return {
				stage: 'BeautyResults',
				pointChange: input.pointChange ?? {},
				storyPointChange: {},
				beautyPointChange: input.beautyPointChange ?? input.pointChange ?? {}
			};
		case 'StellaReveal':
			return {
				stage: 'StellaReveal',
				pointChange: input.pointChange ?? {},
				storyPointChange: {},
				beautyPointChange: {}
			};
		case 'StellaResults':
			return {
				stage: 'StellaResults',
				pointChange: input.pointChange ?? {},
				storyPointChange: {},
				beautyPointChange: {}
			};
		default:
			return null;
	}
}

export function displayedDeltaScores({
	current,
	previous,
	isPreviousResultsMode,
	displayMode = talespinDeltaDisplayMode()
}: {
	current: DeltaScores | null;
	previous: DeltaScores | null;
	isPreviousResultsMode: boolean;
	displayMode?: DeltaDisplayMode;
}): DeltaScores | null {
	if (current) return current;
	if (displayMode === 'only_in_previous_results' && !isPreviousResultsMode) return null;
	return previous;
}
