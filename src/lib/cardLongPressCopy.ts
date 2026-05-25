import { cardImageUrl } from '$lib/cardImageUrls';

const HOLD_MS = 600;

export function longPressCardCopy(
	node: HTMLElement,
	{ card, enabled, onCopy }: { card: string; enabled: boolean; onCopy?: (url: string) => void }
) {
	let timer: number | undefined;
	let copied = false;
	let qualified = false;
	let latestCard = card;
	let latestEnabled = enabled;
	let latestOnCopy = onCopy;

	function clear() {
		if (timer) {
			window.clearTimeout(timer);
			timer = undefined;
		}
	}

	function start(event: PointerEvent) {
		if (!latestEnabled || event.button !== 0) return;
		copied = false;
		qualified = false;
		clear();
		timer = window.setTimeout(() => {
			qualified = true;
			timer = undefined;
		}, HOLD_MS);
	}

	function release() {
		clear();
		if (!qualified) return;
		copied = true;
		qualified = false;
		latestOnCopy?.(cardImageUrl(latestCard));
	}

	function cancel() {
		clear();
		qualified = false;
	}

	function click(event: MouseEvent) {
		if (!copied) return;
		event.preventDefault();
		event.stopPropagation();
		copied = false;
	}

	node.addEventListener('pointerdown', start);
	node.addEventListener('pointerup', release);
	node.addEventListener('pointercancel', cancel);
	node.addEventListener('pointerleave', cancel);
	node.addEventListener('click', click, true);

	return {
		update(next: { card: string; enabled: boolean; onCopy?: (url: string) => void }) {
			latestCard = next.card;
			latestEnabled = next.enabled;
			latestOnCopy = next.onCopy;
			cancel();
		},
		destroy() {
			cancel();
			node.removeEventListener('pointerdown', start);
			node.removeEventListener('pointerup', release);
			node.removeEventListener('pointercancel', cancel);
			node.removeEventListener('pointerleave', cancel);
			node.removeEventListener('click', click, true);
		}
	};
}
