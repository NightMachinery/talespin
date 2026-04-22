import type { PreviousDixitResultsView } from '$lib/types';

export interface PreviousDixitResultsLeaderboardContext {
	stage: 'Results' | 'BeautyResults';
	pointChange: Record<string, number>;
	storyPointChange: Record<string, number>;
	beautyPointChange: Record<string, number>;
	clueRatingOverride: Record<string, number> | null;
}

export function previousDixitResultsLeaderboardContext(
	snapshot: PreviousDixitResultsView | null
): PreviousDixitResultsLeaderboardContext | null {
	if (!snapshot) return null;

	if (snapshot.kind === 'beauty_results') {
		return {
			stage: 'BeautyResults',
			pointChange: snapshot.point_change ?? {},
			storyPointChange: {},
			beautyPointChange: snapshot.point_change ?? {},
			clueRatingOverride:
				Object.keys(snapshot.player_to_clue_rating ?? {}).length > 0
					? snapshot.player_to_clue_rating
					: null
		};
	}

	return {
		stage: 'Results',
		pointChange: snapshot.point_change ?? {},
		storyPointChange: snapshot.storyteller_point_change ?? {},
		beautyPointChange: snapshot.beauty_point_change ?? {},
		clueRatingOverride:
			Object.keys(snapshot.player_to_clue_rating ?? {}).length > 0
				? snapshot.player_to_clue_rating
				: null
	};
}
