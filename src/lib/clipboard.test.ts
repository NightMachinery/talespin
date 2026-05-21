import { beforeEach, describe, expect, test, vi } from 'vitest';

vi.mock('$app/environment', () => ({ browser: true }));

describe('copyTextToClipboard', () => {
	beforeEach(() => {
		vi.resetModules();
		vi.unstubAllGlobals();
		vi.stubGlobal('document', createDocumentStub());
	});

	test('returns true when the native clipboard write succeeds', async () => {
		const writeText = vi.fn().mockResolvedValue(undefined);
		vi.stubGlobal('navigator', { clipboard: { writeText } });

		const { copyTextToClipboard } = await import('./clipboard');

		await expect(copyTextToClipboard('https://example.test/card.jpg')).resolves.toBe(true);
		expect(writeText).toHaveBeenCalledWith('https://example.test/card.jpg');
	});

	test('uses the legacy copy path when the native clipboard is unavailable', async () => {
		vi.stubGlobal('navigator', {});
		const execCommand = vi.spyOn(document, 'execCommand').mockReturnValue(true);

		const { copyTextToClipboard } = await import('./clipboard');

		await expect(copyTextToClipboard('legacy copy')).resolves.toBe(true);
		expect(execCommand).toHaveBeenCalledWith('copy');
		expect(document.querySelector('textarea')).toBeNull();
	});

	test('returns false when native and legacy copy both fail', async () => {
		const writeText = vi.fn().mockRejectedValue(new Error('blocked'));
		vi.stubGlobal('navigator', { clipboard: { writeText } });
		vi.spyOn(document, 'execCommand').mockReturnValue(false);

		const { copyTextToClipboard } = await import('./clipboard');

		await expect(copyTextToClipboard('blocked copy')).resolves.toBe(false);
	});

	test('returns false for blank text', async () => {
		const writeText = vi.fn().mockResolvedValue(undefined);
		vi.stubGlobal('navigator', { clipboard: { writeText } });

		const { copyTextToClipboard } = await import('./clipboard');

		await expect(copyTextToClipboard('   ')).resolves.toBe(false);
		expect(writeText).not.toHaveBeenCalled();
	});
});

function createDocumentStub() {
	const appended: unknown[] = [];

	return {
		body: {
			appendChild: (node: unknown) => appended.push(node),
			removeChild: (node: unknown) => {
				const index = appended.indexOf(node);
				if (index >= 0) appended.splice(index, 1);
			}
		},
		createElement: () => ({
			value: '',
			style: {},
			focus: vi.fn(),
			select: vi.fn()
		}),
		execCommand: vi.fn(),
		querySelector: () => null
	};
}
