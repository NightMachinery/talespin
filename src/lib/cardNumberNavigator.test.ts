import { describe, expect, test } from 'vitest';
import { getCardNumberNavigatorLabels } from './cardNumberNavigator';

describe('getCardNumberNavigatorLabels', () => {
	test('uses explicit overlay labels when provided', () => {
		expect(getCardNumberNavigatorLabels(['card-a', 'card-b'], [4, 9])).toEqual([4, 9]);
	});

	test('falls back to visible card order when labels are missing', () => {
		expect(getCardNumberNavigatorLabels(['card-a', 'card-b', 'card-c'], [7])).toEqual([7, 2, 3]);
	});
});
