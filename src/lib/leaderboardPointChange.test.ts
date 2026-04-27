import { describe, expect, test } from 'vitest';
import { shouldShowPointChange } from './leaderboardPointChange';

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
	});
});
