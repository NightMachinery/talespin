import { describe, expect, test } from 'vitest';
import { shouldRenderPointChange, shouldShowPointChange } from './leaderboardPointChange';

describe('shouldShowPointChange', () => {
	test('shows deltas for previous-results previews even outside results stages', () => {
		expect(shouldShowPointChange('ActiveChooses', true)).toBe(true);
		expect(shouldShowPointChange('PlayersChoose', true)).toBe(true);
	});

	test('keeps live-stage delta visibility unchanged without an override', () => {
		expect(shouldShowPointChange('Results', false)).toBe(true);
		expect(shouldShowPointChange('BeautyResults', false)).toBe(true);
		expect(shouldShowPointChange('StellaReveal', false)).toBe(true);
		expect(shouldShowPointChange('StellaResults', false)).toBe(true);
		expect(shouldShowPointChange('ActiveChooses', false)).toBe(false);
		expect(shouldShowPointChange('PlayersChoose', false)).toBe(false);
	});
});

describe('shouldRenderPointChange', () => {
	test('renders zero deltas whenever result-style delta display is active', () => {
		expect(shouldRenderPointChange('Results', 0)).toBe(true);
		expect(shouldRenderPointChange('BeautyResults', 0)).toBe(true);
		expect(shouldRenderPointChange('ActiveChooses', 0, true)).toBe(true);
		expect(shouldRenderPointChange('PlayersChoose', 0, true)).toBe(true);
	});

	test('does not render deltas during ordinary non-result stages', () => {
		expect(shouldRenderPointChange('ActiveChooses', 5)).toBe(false);
		expect(shouldRenderPointChange('PlayersChoose', 0)).toBe(false);
	});

	test('keeps clue-star standings score-delta-free', () => {
		expect(shouldRenderPointChange('Results', 3, false, 'clue_stars')).toBe(false);
		expect(shouldRenderPointChange('ActiveChooses', 3, true, 'clue_stars')).toBe(false);
	});
});
