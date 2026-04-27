const POINT_CHANGE_STAGES = new Set([
	'Results',
	'BeautyResults',
	'StellaReveal',
	'StellaResults'
]);

export function shouldShowPointChange(stage: string, showPointChangeOverride = false) {
	return showPointChangeOverride || POINT_CHANGE_STAGES.has(stage);
}
