export type BeautyBadgeTier = 'default' | 'gold' | 'silver' | 'bronze';

export interface BeautyBadgeMetadata {
	votes: number;
	rank: number;
	ordinal: string;
	label: string;
	tier: BeautyBadgeTier;
}

function ordinalLabel(rank: number) {
	const mod100 = rank % 100;
	if (mod100 >= 11 && mod100 <= 13) {
		return `${rank}th`;
	}

	switch (rank % 10) {
		case 1:
			return `${rank}st`;
		case 2:
			return `${rank}nd`;
		case 3:
			return `${rank}rd`;
		default:
			return `${rank}th`;
	}
}

function badgeTier(rank: number): BeautyBadgeTier {
	switch (rank) {
		case 1:
			return 'gold';
		case 2:
			return 'silver';
		case 3:
			return 'bronze';
		default:
			return 'default';
	}
}

export function buildBeautyBadgeMetadata(
	beautyVoteTotals: Record<string, number | null | undefined>
): Record<string, BeautyBadgeMetadata> {
	const sortedCards = Object.entries(beautyVoteTotals)
		.filter((entry): entry is [string, number] => typeof entry[1] === 'number' && entry[1] > 0)
		.map(([card, votes]) => ({ card, votes }))
		.sort((a, b) => b.votes - a.votes || a.card.localeCompare(b.card));

	let previousVotes: number | null = null;
	let previousRank = 0;

	return Object.fromEntries(
		sortedCards.map(({ card, votes }, index) => {
			const rank = previousVotes === votes ? previousRank : index + 1;
			const ordinal = ordinalLabel(rank);

			previousVotes = votes;
			previousRank = rank;

			return [
				card,
				{
					votes,
					rank,
					ordinal,
					label: `${ordinal} Beauty: ${votes}`,
					tier: badgeTier(rank)
				}
			];
		})
	);
}
