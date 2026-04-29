import { describe, expect, test } from 'vitest';
import { firstActiveRoundForPlayer, scoreBreakdownsFromSnapshots } from './leaderboard';
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
