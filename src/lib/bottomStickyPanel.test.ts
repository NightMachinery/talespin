import { describe, expect, test } from 'vitest';

import {
	BOTTOM_STICKY_PANEL_ACTION_LAYOUT,
	BOTTOM_STICKY_PANEL_VIEW_PRESENTATION,
	bottomStickyPanelActionTooltip,
	bottomStickyPanelGridStyle,
	bottomStickyPanelVisibleLabel,
	type BottomStickyPanelAction
} from './bottomStickyPanel';

describe('bottom sticky panel defaults', () => {
	test('keeps view and action defaults centralized for hand edits', () => {
		expect(BOTTOM_STICKY_PANEL_VIEW_PRESENTATION).toBe('text');
		expect(BOTTOM_STICKY_PANEL_ACTION_LAYOUT).toBe('row');
	});

	test('builds a single-row grid style from the action count', () => {
		expect(bottomStickyPanelGridStyle(3)).toBe('grid-template-columns: repeat(3, minmax(0, 1fr));');
	});

	test('does not build invalid grid styles for empty action groups', () => {
		expect(bottomStickyPanelGridStyle(0)).toBe('');
	});

	test('uses short labels for compact action buttons when available', () => {
		const action: BottomStickyPanelAction = {
			label: 'Auto-observerify offline players',
			shortLabel: 'Auto-obs',
			onClick: () => undefined
		};

		expect(bottomStickyPanelVisibleLabel(action)).toBe('Auto-obs');
	});

	test('uses compact visible labels for reset-style rectangular buttons', () => {
		const action: BottomStickyPanelAction = {
			label: 'Reset clue',
			shortLabel: 'Reset',
			tooltip: 'Reset clue and return to storyteller choosing',
			iconOnly: true,
			onClick: () => undefined
		};

		expect(bottomStickyPanelVisibleLabel(action)).toBe('Reset');
		expect(bottomStickyPanelActionTooltip(action)).toBe(
			'Reset clue and return to storyteller choosing'
		);
	});

	test('falls back to the full label when a compact action has no short label', () => {
		const action: BottomStickyPanelAction = {
			label: 'Reset board',
			iconOnly: true,
			onClick: () => undefined
		};

		expect(bottomStickyPanelVisibleLabel(action)).toBe('Reset board');
	});
});
