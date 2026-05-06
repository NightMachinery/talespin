import { http_host } from '$lib/gameServer';

const HOLD_MS = 600;

export function cardImageUrl(card: string) {
	return `${http_host}/cards/${card}`;
}

export function longPressCardCopy(
	node: HTMLElement,
	{ card, enabled, onCopy }: { card: string; enabled: boolean; onCopy?: (url: string) => void }
) {
	let timer: number | undefined;
	let copied = false;
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
		clear();
		timer = window.setTimeout(() => {
			copied = true;
			latestOnCopy?.(cardImageUrl(latestCard));
		}, HOLD_MS);
	}

	function stop() {
		clear();
	}

	function click(event: MouseEvent) {
		if (!copied) return;
		event.preventDefault();
		event.stopPropagation();
		copied = false;
	}

	node.addEventListener('pointerdown', start);
	node.addEventListener('pointerup', stop);
	node.addEventListener('pointercancel', stop);
	node.addEventListener('pointerleave', stop);
	node.addEventListener('click', click, true);

	return {
		update(next: { card: string; enabled: boolean; onCopy?: (url: string) => void }) {
			latestCard = next.card;
			latestEnabled = next.enabled;
			latestOnCopy = next.onCopy;
			clear();
		},
		destroy() {
			clear();
			node.removeEventListener('pointerdown', start);
			node.removeEventListener('pointerup', stop);
			node.removeEventListener('pointercancel', stop);
			node.removeEventListener('pointerleave', stop);
			node.removeEventListener('click', click, true);
		}
	};
}
