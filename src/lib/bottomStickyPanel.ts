export type BottomStickyPanelViewPresentation = 'icon' | 'text';
export type BottomStickyPanelActionLayout = 'stack' | 'row';
export type BottomStickyPanelActionIcon =
	| 'rotate-ccw'
	| 'refresh-cw'
	| 'shuffle'
	| 'skip-forward'
	| 'fast-forward'
	| 'user-x'
	| 'play';

export type BottomStickyPanelAction = {
	label: string;
	shortLabel?: string;
	icon?: BottomStickyPanelActionIcon;
	iconOnly?: boolean;
	tooltip?: string;
	confirmMessage?: string;
	disabled?: boolean;
	onClick: () => void;
};

export const BOTTOM_STICKY_PANEL_VIEW_PRESENTATION: BottomStickyPanelViewPresentation = 'text';
export const BOTTOM_STICKY_PANEL_ACTION_LAYOUT: BottomStickyPanelActionLayout = 'row';

export function bottomStickyPanelGridStyle(itemCount: number) {
	if (itemCount <= 0) return '';
	return `grid-template-columns: repeat(${itemCount}, minmax(0, 1fr));`;
}

export function bottomStickyPanelVisibleLabel(action: BottomStickyPanelAction) {
	return action.shortLabel ?? action.label;
}

export function bottomStickyPanelActionTooltip(action: BottomStickyPanelAction) {
	return action.tooltip ?? action.label;
}
