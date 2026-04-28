import { describe, expect, test } from 'vitest';

import { resolvePlayersChooseSelection } from './playersChooseSelection';

describe('resolvePlayersChooseSelection', () => {
	test('uses server selections before draft selections', () => {
		const selection = resolvePlayersChooseSelection({
			serverSelectedCards: ['server-card'],
			draftSelectedCards: ['draft-card'],
			displayImages: ['server-card', 'draft-card'],
			maxSelections: 2
		});

		expect(selection).toEqual(['server-card']);
	});

	test('uses draft selections when server selections are empty', () => {
		const selection = resolvePlayersChooseSelection({
			serverSelectedCards: [],
			draftSelectedCards: ['draft-card'],
			displayImages: ['server-card', 'draft-card'],
			maxSelections: 2
		});

		expect(selection).toEqual(['draft-card']);
	});

	test('filters restored selections to the current hand and nomination limit', () => {
		const selection = resolvePlayersChooseSelection({
			serverSelectedCards: [],
			draftSelectedCards: ['old-card', 'first-card', 'second-card', 'third-card'],
			displayImages: ['first-card', 'second-card', 'third-card'],
			maxSelections: 2
		});

		expect(selection).toEqual(['second-card', 'third-card']);
	});
});
