import { describe, expect, test, vi } from 'vitest';

vi.mock('$lib/gameServer', () => ({ http_host: 'https://talespin.example' }));

describe('card image URL helpers', () => {
	test('builds compressed and original card URLs', async () => {
		const { cardImageUrl, originalCardImageUrl } = await import('./cardImageUrls');

		expect(cardImageUrl('abc123')).toBe('https://talespin.example/cards/abc123');
		expect(originalCardImageUrl('abc123')).toBe('https://talespin.example/cards/abc123_original');
	});

	test('builds stable original download filenames', async () => {
		const { originalCardDownloadFilename } = await import('./cardImageUrls');

		expect(originalCardDownloadFilename('abc123')).toBe('abc123_original');
	});
});
