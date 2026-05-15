import { describe, expect, test } from 'vitest';

import {
	BOTTOM_STICKY_PANEL_ACTION_LAYOUT,
	BOTTOM_STICKY_PANEL_VIEW_PRESENTATION,
	bottomStickyPanelGridStyle
} from './bottomStickyPanel';

describe('bottom sticky panel defaults', () => {
	test('keeps view and action defaults centralized for hand edits', () => {
		expect(BOTTOM_STICKY_PANEL_VIEW_PRESENTATION).toBe('icon');
		expect(BOTTOM_STICKY_PANEL_ACTION_LAYOUT).toBe('stack');
	});

	test('builds a single-row grid style from the action count', () => {
		expect(bottomStickyPanelGridStyle(3)).toBe('grid-template-columns: repeat(3, minmax(0, 1fr));');
	});

	test('does not build invalid grid styles for empty action groups', () => {
		expect(bottomStickyPanelGridStyle(0)).toBe('');
	});
});
