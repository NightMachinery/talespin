import { describe, expect, test } from 'vitest';
import {
	currentRoundDeltaScores,
	displayedDeltaScores,
	previousRoundDeltaScores,
	type DeltaScores
} from './deltaScores';
import type { PreviousDixitResultsView } from './types';

const previousResultsSnapshot: PreviousDixitResultsView = {
	kind: 'results',
	center_cards: [],
	player_to_votes: {},
	player_to_beauty_votes: {},
	player_to_clue_rating: {},
	clue_rating_average: null,
	clue_rating_count: 0,
	clue_rating_bonus: 0,
	player_to_current_cards: {},
	active_card: '',
	beauty_results_display_mode: 'combined',
	point_change: { Alice: 5, Bob: 1 },
	storyteller_point_change: { Alice: 3 },
	beauty_point_change: { Alice: 2, Bob: 1 },
	beauty_vote_totals: {},
	beauty_winning_cards: []
};

const previousBeautyResultsSnapshot: PreviousDixitResultsView = {
	kind: 'beauty_results',
	center_cards: [],
	player_to_beauty_votes: {},
	player_to_clue_rating: {},
	clue_rating_average: null,
	clue_rating_count: 0,
	clue_rating_bonus: 0,
	player_to_current_cards: {},
	point_change: { Bob: 2 },
	beauty_vote_totals: {},
	beauty_winning_cards: []
};

const previousScores: DeltaScores = {
	stage: 'Results',
	pointChange: { Alice: 3 },
	storyPointChange: { Alice: 3 },
	beautyPointChange: {}
};

const currentScores: DeltaScores = {
	stage: 'BeautyResults',
	pointChange: { Bob: 2 },
	storyPointChange: {},
	beautyPointChange: { Bob: 2 }
};

describe('previousRoundDeltaScores', () => {
	test('extracts total, story, and beauty deltas from previous results snapshots', () => {
		expect(previousRoundDeltaScores(previousResultsSnapshot)).toEqual({
			stage: 'Results',
			pointChange: { Alice: 5, Bob: 1 },
			storyPointChange: { Alice: 3 },
			beautyPointChange: { Alice: 2, Bob: 1 }
		});
	});

	test('extracts beauty deltas from previous beauty-results snapshots', () => {
		expect(previousRoundDeltaScores(previousBeautyResultsSnapshot)).toEqual({
			stage: 'BeautyResults',
			pointChange: { Bob: 2 },
			storyPointChange: {},
			beautyPointChange: { Bob: 2 }
		});
	});
});

describe('currentRoundDeltaScores', () => {
	test('extracts current Dixit results deltas', () => {
		expect(
			currentRoundDeltaScores({
				stage: 'Results',
				pointChange: { Alice: 5 },
				storytellerPointChange: { Alice: 3 },
				beautyPointChange: { Alice: 2 }
			})
		).toEqual({
			stage: 'Results',
			pointChange: { Alice: 5 },
			storyPointChange: { Alice: 3 },
			beautyPointChange: { Alice: 2 }
		});
	});

	test('extracts current beauty-results deltas as beauty-only deltas', () => {
		expect(
			currentRoundDeltaScores({
				stage: 'BeautyResults',
				pointChange: { Bob: 2 },
				storytellerPointChange: { Alice: 3 },
				beautyPointChange: { Bob: 2 }
			})
		).toEqual({
			stage: 'BeautyResults',
			pointChange: { Bob: 2 },
			storyPointChange: {},
			beautyPointChange: { Bob: 2 }
		});
	});

	test('extracts current Stella deltas as total-only deltas', () => {
		expect(
			currentRoundDeltaScores({
				stage: 'StellaResults',
				pointChange: { Scout: 4 },
				storytellerPointChange: {},
				beautyPointChange: {}
			})
		).toEqual({
			stage: 'StellaResults',
			pointChange: { Scout: 4 },
			storyPointChange: {},
			beautyPointChange: {}
		});
	});

	test('returns null outside result stages', () => {
		expect(
			currentRoundDeltaScores({
				stage: 'PlayersChoose',
				pointChange: { Alice: 3 },
				storytellerPointChange: { Alice: 3 },
				beautyPointChange: {}
			})
		).toBeNull();
	});
});

describe('displayedDeltaScores', () => {
	test('always returns current scores when current scores are available', () => {
		expect(
			displayedDeltaScores({
				current: currentScores,
				previous: previousScores,
				isPreviousResultsMode: false,
				displayMode: 'only_in_previous_results'
			})
		).toBe(currentScores);
	});

	test('falls back to previous scores by default', () => {
		expect(
			displayedDeltaScores({
				current: null,
				previous: previousScores,
				isPreviousResultsMode: false,
				displayMode: 'always'
			})
		).toBe(previousScores);
	});

	test('only falls back to previous scores in previous-results mode when configured', () => {
		expect(
			displayedDeltaScores({
				current: null,
				previous: previousScores,
				isPreviousResultsMode: false,
				displayMode: 'only_in_previous_results'
			})
		).toBeNull();
		expect(
			displayedDeltaScores({
				current: null,
				previous: previousScores,
				isPreviousResultsMode: true,
				displayMode: 'only_in_previous_results'
			})
		).toBe(previousScores);
	});
});
