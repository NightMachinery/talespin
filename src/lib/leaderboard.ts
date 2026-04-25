import { writable } from 'svelte/store';
import type { LeaderboardRoundHistoryEntry, LeaderboardViewMode } from '$lib/types';

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

export interface CurrentLeaderboardScoreEntry {
	name: string;
	total: number;
	beauty: number;
	isPlayer: boolean;
	hasScore: boolean;
}

export const leaderboardRoundHistory = writable<LeaderboardRoundHistoryEntry[]>([]);

export function scoreBreakdown(total: number, beauty: number): LeaderboardScoreBreakdown {
	return {
		total,
		story: total - beauty,
		beauty
	};
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

export function sinceJoinedScoreBreakdowns(
	entries: CurrentLeaderboardScoreEntry[],
	history: LeaderboardRoundHistoryEntry[],
	startRound: number
) {
	const simulatedScores = new Map<string, { total: number; beauty: number }>();
	const currentEntryByName = new Map(entries.map((entry) => [entry.name, entry]));

	for (const round of sortedLeaderboardHistory(history)) {
		if (round.round_num < startRound) continue;

		const participants = new Set([
			...round.active_players,
			...Object.keys(round.total_deltas),
			...Object.keys(round.beauty_deltas)
		]);
		const simulatedFloor = simulatedFloorForRound(round.active_players, simulatedScores);
		for (const participant of participants) {
			if (!simulatedScores.has(participant)) {
				simulatedScores.set(participant, { total: simulatedFloor, beauty: 0 });
			}
		}

		for (const participant of participants) {
			const current = simulatedScores.get(participant);
			if (!current) continue;
			current.total = applySignedScoreDelta(current.total, round.total_deltas[participant] ?? 0);
			current.beauty = applySignedScoreDelta(current.beauty, round.beauty_deltas[participant] ?? 0);
		}
	}

	const liveFloor = liveSimulatedFloor(entries, simulatedScores);
	for (const entry of entries) {
		if (!entry.hasScore || simulatedScores.has(entry.name)) continue;
		simulatedScores.set(entry.name, { total: entry.isPlayer ? liveFloor : 0, beauty: 0 });
	}

	const breakdowns = new Map<string, LeaderboardScoreBreakdown>();
	for (const entry of entries) {
		if (!entry.hasScore) continue;
		const simulated = simulatedScores.get(entry.name) ?? { total: 0, beauty: 0 };
		breakdowns.set(entry.name, scoreBreakdown(simulated.total, simulated.beauty));
	}

	for (const [name, simulated] of simulatedScores) {
		if (!currentEntryByName.has(name)) continue;
		breakdowns.set(name, scoreBreakdown(simulated.total, simulated.beauty));
	}

	return breakdowns;
}

function sortedLeaderboardHistory(history: LeaderboardRoundHistoryEntry[]) {
	return [...history].sort((a, b) => a.round_num - b.round_num);
}

function simulatedFloorForRound(
	activePlayers: string[],
	scores: Map<string, { total: number; beauty: number }>
) {
	const existingActiveTotals = activePlayers
		.map((playerName) => scores.get(playerName)?.total)
		.filter((value): value is number => typeof value === 'number');
	return existingActiveTotals.length > 0 ? Math.min(...existingActiveTotals) : 0;
}

function liveSimulatedFloor(
	entries: CurrentLeaderboardScoreEntry[],
	scores: Map<string, { total: number; beauty: number }>
) {
	const activeTotals = entries
		.filter((entry) => entry.isPlayer)
		.map((entry) => scores.get(entry.name)?.total)
		.filter((value): value is number => typeof value === 'number');
	return activeTotals.length > 0 ? Math.min(...activeTotals) : 0;
}

function applySignedScoreDelta(score: number, delta: number) {
	return Math.max(0, score + delta);
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
