import { writable } from 'svelte/store';
import type {
	LeaderboardRoundHistoryEntry,
	LeaderboardScoreSnapshot,
	LeaderboardViewMode
} from '$lib/types';

export interface LeaderboardScoreBreakdown {
	total: number;
	story: number;
	beauty: number;
}

export interface RankedLeaderboardEntry {
	name: string;
	breakdown: LeaderboardScoreBreakdown;
	clueStars: number | null;
	rank: number;
	isTopScore: boolean;
}

export const leaderboardRoundHistory = writable<LeaderboardRoundHistoryEntry[]>([]);
export const leaderboardSinceJoinedScoresByRound = writable<
	Record<number, Record<string, LeaderboardScoreSnapshot>>
>({});

export function scoreBreakdown(total: number, beauty: number): LeaderboardScoreBreakdown {
	return {
		total,
		story: total - beauty,
		beauty
	};
}

export function scoreBreakdownsFromSnapshots(
	scores: Record<string, LeaderboardScoreSnapshot> | undefined
) {
	return new Map(
		Object.entries(scores ?? {}).map(([playerName, score]) => [
			playerName,
			scoreBreakdown(score.total, score.beauty)
		])
	);
}

export function firstActiveRoundForPlayer(
	history: LeaderboardRoundHistoryEntry[],
	playerName: string
) {
	for (const round of sortedLeaderboardHistory(history)) {
		if (round.active_players.includes(playerName)) {
			return round.round_num;
		}
	}
	return null;
}

function sortedLeaderboardHistory(history: LeaderboardRoundHistoryEntry[]) {
	return [...history].sort((a, b) => a.round_num - b.round_num);
}

export function leaderboardModeLabel(mode: LeaderboardViewMode) {
	switch (mode) {
		case 'story_only':
			return 'Story Only';
		case 'beauty_only':
			return 'Beauty Only';
		case 'combined':
			return 'Combined';
		case 'clue_stars':
			return 'Clue Stars';
		case 'total':
		default:
			return 'Total';
	}
}

export function leaderboardModeValue(
	mode: LeaderboardViewMode,
	breakdown: LeaderboardScoreBreakdown,
	clueStars: number | null = null
) {
	switch (mode) {
		case 'story_only':
			return breakdown.story;
		case 'beauty_only':
			return breakdown.beauty;
		case 'clue_stars':
			return clueStars;
		case 'combined':
		case 'total':
		default:
			return breakdown.total;
	}
}

export function rankEntriesByMode(
	entries: Array<{
		name: string;
		breakdown: LeaderboardScoreBreakdown;
		clueStars?: number | null;
	}>,
	mode: LeaderboardViewMode
): RankedLeaderboardEntry[] {
	const sortedEntries = [...entries].sort((a, b) => {
		const bValue = leaderboardModeValue(mode, b.breakdown, b.clueStars ?? null);
		const aValue = leaderboardModeValue(mode, a.breakdown, a.clueStars ?? null);
		if (mode === 'clue_stars') {
			if (bValue === null && aValue !== null) return -1;
			if (aValue === null && bValue !== null) return 1;
			if (aValue !== null && bValue !== null) {
				const valueDiff = bValue - aValue;
				if (valueDiff !== 0) return valueDiff;
			}
		} else {
			const valueDiff = (bValue ?? 0) - (aValue ?? 0);
			if (valueDiff !== 0) return valueDiff;
		}
		if (b.breakdown.total !== a.breakdown.total) return b.breakdown.total - a.breakdown.total;
		return a.name.localeCompare(b.name);
	});
	const topValue =
		sortedEntries.length > 0
			? leaderboardModeValue(mode, sortedEntries[0].breakdown, sortedEntries[0].clueStars ?? null)
			: null;

	let previousValue: number | null = null;
	let previousRank = 0;

	return sortedEntries.map(({ name, breakdown, clueStars }, index) => {
		const currentValue = leaderboardModeValue(mode, breakdown, clueStars ?? null);
		const rank = previousValue === currentValue ? previousRank : index + 1;
		previousValue = currentValue;
		previousRank = rank;
		return {
			name,
			breakdown,
			clueStars: clueStars ?? null,
			rank,
			isTopScore: topValue !== null && currentValue === topValue
		};
	});
}

function digitCount(value: number) {
	return Math.max(1, `${Math.abs(value)}`.length);
}

export function leaderboardDigitWidths(entries: Array<{ breakdown: LeaderboardScoreBreakdown }>) {
	return entries.reduce(
		(widths, entry) => ({
			total: Math.max(widths.total, digitCount(entry.breakdown.total)),
			story: Math.max(widths.story, digitCount(entry.breakdown.story)),
			beauty: Math.max(widths.beauty, digitCount(entry.breakdown.beauty))
		}),
		{ total: 1, story: 1, beauty: 1 }
	);
}

export function isLeaderboardModeSupported(
	mode: LeaderboardViewMode,
	supportsBeautyModes: boolean,
	supportsClueStarsMode: boolean
) {
	if (mode === 'combined' || mode === 'beauty_only' || mode === 'story_only') {
		return supportsBeautyModes;
	}
	if (mode === 'clue_stars') {
		return supportsClueStarsMode;
	}
	return true;
}

export function resolveLeaderboardMode(
	mode: LeaderboardViewMode,
	supportsBeautyModes: boolean,
	supportsClueStarsMode: boolean
): LeaderboardViewMode {
	if (isLeaderboardModeSupported(mode, supportsBeautyModes, supportsClueStarsMode)) {
		return mode;
	}
	return supportsBeautyModes ? 'combined' : 'total';
}
