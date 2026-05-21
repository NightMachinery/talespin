import { afterEach, beforeEach, describe, expect, test, vi } from 'vitest';

vi.mock('$lib/gameServer', () => ({ http_host: 'https://talespin.example' }));

describe('longPressCardCopy', () => {
	beforeEach(() => {
		vi.useFakeTimers();
		vi.stubGlobal('window', {
			setTimeout: globalThis.setTimeout,
			clearTimeout: globalThis.clearTimeout
		});
	});

	afterEach(() => {
		vi.useRealTimers();
		vi.unstubAllGlobals();
	});

	test('does not copy before the hold threshold', async () => {
		const { longPressCardCopy } = await import('./cardLongPressCopy');
		const node = new EventTarget() as HTMLElement;
		const onCopy = vi.fn();
		longPressCardCopy(node, { card: '12.jpeg', enabled: true, onCopy });

		node.dispatchEvent(pointerEvent('pointerdown'));
		vi.advanceTimersByTime(599);
		node.dispatchEvent(pointerEvent('pointerup'));

		expect(onCopy).not.toHaveBeenCalled();
	});

	test('copies the card URL on pointerup after the hold threshold', async () => {
		const { longPressCardCopy } = await import('./cardLongPressCopy');
		const node = new EventTarget() as HTMLElement;
		const onCopy = vi.fn();
		longPressCardCopy(node, { card: '12.jpeg', enabled: true, onCopy });

		node.dispatchEvent(pointerEvent('pointerdown'));
		vi.advanceTimersByTime(600);

		expect(onCopy).not.toHaveBeenCalled();

		node.dispatchEvent(pointerEvent('pointerup'));

		expect(onCopy).toHaveBeenCalledWith('https://talespin.example/cards/12.jpeg');
	});

	test('cancelling before the hold threshold prevents copy', async () => {
		const { longPressCardCopy } = await import('./cardLongPressCopy');
		const node = new EventTarget() as HTMLElement;
		const onCopy = vi.fn();
		longPressCardCopy(node, { card: '12.jpeg', enabled: true, onCopy });

		node.dispatchEvent(pointerEvent('pointerdown'));
		vi.advanceTimersByTime(300);
		node.dispatchEvent(pointerEvent('pointercancel'));
		vi.advanceTimersByTime(300);
		node.dispatchEvent(pointerEvent('pointerup'));

		expect(onCopy).not.toHaveBeenCalled();
	});

	test('a qualified hold suppresses the following click', async () => {
		const { longPressCardCopy } = await import('./cardLongPressCopy');
		const node = new EventTarget() as HTMLElement;
		longPressCardCopy(node, { card: '12.jpeg', enabled: true, onCopy: vi.fn() });

		node.dispatchEvent(pointerEvent('pointerdown'));
		vi.advanceTimersByTime(600);
		node.dispatchEvent(pointerEvent('pointerup'));

		const click = new Event('click', { bubbles: true, cancelable: true });
		const stopPropagation = vi.spyOn(click, 'stopPropagation');

		node.dispatchEvent(click);

		expect(click.defaultPrevented).toBe(true);
		expect(stopPropagation).toHaveBeenCalled();
	});
});

function pointerEvent(type: string) {
	const event = new Event(type, { bubbles: true, cancelable: true });
	Object.defineProperty(event, 'button', { value: 0 });
	return event;
}
