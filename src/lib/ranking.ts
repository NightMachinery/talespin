import type { PlayerInfo } from '$lib/types';

export interface RankedPlayerEntry {
	name: string;
	points: number;
	rank: number;
	isTopScore: boolean;
}

export function rankPlayersByPoints(players: { [key: string]: PlayerInfo }): RankedPlayerEntry[] {
	const sortedPlayerNames = Object.keys(players).sort((a, b) => players[b].points - players[a].points);
	const topPoints = sortedPlayerNames.length > 0 ? players[sortedPlayerNames[0]].points : null;

	let previousPoints: number | null = null;
	let previousRank = 0;

	return sortedPlayerNames.map((name, index) => {
		const points = players[name].points;
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
