export type BottomStickyPanelViewPresentation = 'icon' | 'text';
export type BottomStickyPanelActionLayout = 'stack' | 'row';

export const BOTTOM_STICKY_PANEL_VIEW_PRESENTATION: BottomStickyPanelViewPresentation = 'icon';
export const BOTTOM_STICKY_PANEL_ACTION_LAYOUT: BottomStickyPanelActionLayout = 'stack';

export function bottomStickyPanelGridStyle(itemCount: number) {
	if (itemCount <= 0) return '';
	return `grid-template-columns: repeat(${itemCount}, minmax(0, 1fr));`;
}
