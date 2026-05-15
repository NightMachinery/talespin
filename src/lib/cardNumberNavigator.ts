export function buildCardNumberNavigatorTargetId(scope: string, label: number) {
	return `card-number-navigator-${scope}-${label}`;
}

export function getCardNumberNavigatorLabels(
	displayImages: string[],
	explicitLabels: number[] = []
) {
	return displayImages.map((_, index) => explicitLabels[index] ?? index + 1);
}

export const CARD_NUMBER_NAVIGATOR_SCROLL_MARGIN_TOP = '10rem';
