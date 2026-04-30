const POINT_CHANGE_STAGES = new Set(['Results', 'BeautyResults', 'StellaReveal', 'StellaResults']);

export type PointChangeLeaderboardViewMode =
	| 'total'
	| 'story_only'
	| 'beauty_only'
	| 'combined'
	| 'clue_stars';

export function shouldShowPointChange(stage: string, showPointChangeOverride = false) {
	return showPointChangeOverride || POINT_CHANGE_STAGES.has(stage);
}

export function shouldRenderPointChange(
	stage: string,
	_delta: number | null,
	showPointChangeOverride = false,
	leaderboardViewMode: PointChangeLeaderboardViewMode = 'total'
) {
	return (
		leaderboardViewMode !== 'clue_stars' && shouldShowPointChange(stage, showPointChangeOverride)
	);
}
