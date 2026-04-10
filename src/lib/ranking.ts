import type { PlayerInfo } from '$lib/types';

export interface RankedPlayerEntry {
	name: string;
	points: number;
	rank: number;
	isTopScore: boolean;
}

export function rankEntriesByPoints(entries: Array<{ name: string; points: number }>): RankedPlayerEntry[] {
	const sortedEntries = [...entries].sort(
		(a, b) => b.points - a.points || a.name.localeCompare(b.name)
	);
	const topPoints = sortedEntries.length > 0 ? sortedEntries[0].points : null;

	let previousPoints: number | null = null;
	let previousRank = 0;

	return sortedEntries.map(({ name, points }, index) => {
		const rank = previousPoints === points ? previousRank : index + 1;

		previousPoints = points;
		previousRank = rank;

		return {
			name,
			points,
			rank,
			isTopScore: topPoints !== null && points === topPoints
		};
	});
}

export function rankPlayersByPoints(players: { [key: string]: PlayerInfo }): RankedPlayerEntry[] {
	return rankEntriesByPoints(
		Object.entries(players).map(([name, info]) => ({
			name,
			points: info.points
		}))
	);
}
