import { describe, expect, test } from 'vitest';
import {
	firstActiveRoundForPlayer,
	rankEntriesByMode,
	rankLeaderboardEntries,
	scoreBreakdownsFromSnapshots
} from './leaderboard';
import type { LeaderboardRoundHistoryEntry } from './types';

function round(round_num: number, active_players: string[]): LeaderboardRoundHistoryEntry {
	return {
		round_num,
		active_players,
		total_deltas: {},
		beauty_deltas: {},
		total_after_round: {},
		beauty_total_after_round: {}
	};
}

describe('firstActiveRoundForPlayer', () => {
	test('returns the first active round even when history arrives out of order', () => {
		const history = [round(5, ['Alice', 'Bob']), round(3, ['Bob']), round(4, ['Alice'])];

		expect(firstActiveRoundForPlayer(history, 'Alice')).toBe(4);
		expect(firstActiveRoundForPlayer(history, 'Bob')).toBe(3);
	});

	test('returns null when the member has not been active in recorded history', () => {
		expect(firstActiveRoundForPlayer([round(1, ['Alice'])], 'PendingObserver')).toBeNull();
	});

	test('returns the current round when the member first became active in the live round', () => {
		expect(
			firstActiveRoundForPlayer([round(1, ['Alice']), round(2, ['Alice'])], 'Bob', {
				roundNum: 3,
				activePlayers: ['Alice', 'Bob']
			})
		).toBe(3);
	});

	test('prefers historical first active round over the current live round', () => {
		expect(
			firstActiveRoundForPlayer([round(2, ['Bob'])], 'Bob', {
				roundNum: 5,
				activePlayers: ['Alice', 'Bob']
			})
		).toBe(2);
	});
});

describe('scoreBreakdownsFromSnapshots', () => {
	test('converts server-computed total and beauty snapshots to display breakdowns', () => {
		const breakdowns = scoreBreakdownsFromSnapshots({
			Alice: { total: 10, beauty: 3 },
			Bob: { total: 5, beauty: 0 }
		});

		expect(breakdowns.get('Alice')).toEqual({ total: 10, story: 7, beauty: 3 });
		expect(breakdowns.get('Bob')).toEqual({ total: 5, story: 5, beauty: 0 });
	});

	test('treats a missing server map as an empty display map', () => {
		expect(scoreBreakdownsFromSnapshots(undefined).size).toBe(0);
	});
});

describe('rankLeaderboardEntries', () => {
	test('assigns the same competition rank to equal clue-star scores', () => {
		const ranked = rankEntriesByMode(
			[
				{ name: 'Alice', breakdown: { total: 7, story: 7, beauty: 0 }, clueStars: 4.5 },
				{ name: 'Bob', breakdown: { total: 9, story: 9, beauty: 0 }, clueStars: 4.5 },
				{ name: 'Carol', breakdown: { total: 12, story: 12, beauty: 0 }, clueStars: 3 },
				{ name: 'Drew', breakdown: { total: 2, story: 2, beauty: 0 }, clueStars: null }
			],
			'clue_stars'
		);

		expect(ranked.map(({ name, rank }) => ({ name, rank }))).toEqual([
			{ name: 'Bob', rank: 1 },
			{ name: 'Alice', rank: 1 },
			{ name: 'Carol', rank: 3 },
			{ name: 'Drew', rank: 4 }
		]);
	});

	test('supports future leaderboard rankers through a shared helper', () => {
		const ranked = rankLeaderboardEntries(
			[
				{ name: 'B', value: 8 },
				{ name: 'A', value: 8 },
				{ name: 'C', value: 3 }
			],
			(entry) => entry.value
		);

		expect(ranked.map(({ name, rank, isTopScore }) => ({ name, rank, isTopScore }))).toEqual([
			{ name: 'A', rank: 1, isTopScore: true },
			{ name: 'B', rank: 1, isTopScore: true },
			{ name: 'C', rank: 3, isTopScore: false }
		]);
	});
});
