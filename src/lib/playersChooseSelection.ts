type ResolvePlayersChooseSelectionArgs = {
	serverSelectedCards: string[];
	draftSelectedCards: string[];
	displayImages: string[];
	maxSelections: number;
};

type PlayersChooseDraftKeyArgs = {
	roomCode: string;
	name: string;
	roundNum: number;
	activePlayer: string;
};

export function resolvePlayersChooseSelection({
	serverSelectedCards,
	draftSelectedCards,
	displayImages,
	maxSelections
}: ResolvePlayersChooseSelectionArgs): string[] {
	const source = serverSelectedCards.length > 0 ? serverSelectedCards : draftSelectedCards;
	const displayImageSet = new Set(displayImages);
	const filtered = source.filter((card) => displayImageSet.has(card));
	const limit = Math.max(1, maxSelections);

	return filtered.slice(Math.max(0, filtered.length - limit));
}

export function playersChooseDraftKey({
	roomCode,
	name,
	roundNum,
	activePlayer
}: PlayersChooseDraftKeyArgs): string {
	return `players_choose_selection:${roomCode}:${name}:${roundNum}:${activePlayer}`;
}

export function readPlayersChooseDraft(storage: Storage | undefined, key: string): string[] {
	if (!storage || key === '') return [];

	try {
		const raw = storage.getItem(key);
		if (!raw) return [];
		const parsed = JSON.parse(raw);
		if (!Array.isArray(parsed)) return [];

		return parsed.filter((value): value is string => typeof value === 'string');
	} catch {
		return [];
	}
}

export function writePlayersChooseDraft(
	storage: Storage | undefined,
	key: string,
	selectedCards: string[]
) {
	if (!storage || key === '') return;

	if (selectedCards.length === 0) {
		storage.removeItem(key);
		return;
	}

	storage.setItem(key, JSON.stringify(selectedCards));
}
