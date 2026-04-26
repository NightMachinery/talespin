import { describe, expect, test } from 'vitest';
import { sinceJoinedScoreBreakdowns, type CurrentLeaderboardScoreEntry } from './leaderboard';
import type { LeaderboardRoundHistoryEntry } from './types';

function entry(
	name: string,
	total: number,
	beauty = 0,
	isPlayer = true
): CurrentLeaderboardScoreEntry {
	return { name, total, beauty, isPlayer, hasScore: true };
}

function round(
	round_num: number,
	active_players: string[],
	total_deltas: Record<string, number>,
	beauty_deltas: Record<string, number>,
	total_after_round: Record<string, number>,
	beauty_total_after_round: Record<string, number>
): LeaderboardRoundHistoryEntry {
	return {
		round_num,
		active_players,
		total_deltas,
		beauty_deltas,
		total_after_round,
		beauty_total_after_round
	};
}

describe('sinceJoinedScoreBreakdowns', () => {
	test('replaying from round 1 matches raw cumulative scores for players who become active later', () => {
		const history = [
			round(1, ['Alice'], { Alice: 10 }, {}, { Alice: 10 }, {}),
			round(2, ['Alice'], { Bob: 3 }, {}, { Alice: 10, Bob: 3 }, {}),
			round(3, ['Alice', 'Bob'], { Bob: 2 }, {}, { Alice: 10, Bob: 5 }, {})
		];

		const breakdowns = sinceJoinedScoreBreakdowns(
			[entry('Alice', 10), entry('Bob', 5)],
			history,
			1
		);

		expect(breakdowns.get('Alice')).toEqual({ total: 10, story: 10, beauty: 0 });
		expect(breakdowns.get('Bob')).toEqual({ total: 5, story: 5, beauty: 0 });
	});

	test('does not apply active-player join floors to non-active scored observers', () => {
		const history = [
			round(1, ['Alice'], { Alice: 10 }, {}, { Alice: 10 }, {}),
			round(2, ['Alice'], { Observer: 4 }, {}, { Alice: 10, Observer: 4 }, {})
		];

		const breakdowns = sinceJoinedScoreBreakdowns(
			[entry('Alice', 10), entry('Observer', 4, 0, false)],
			history,
			1
		);

		expect(breakdowns.get('Observer')).toEqual({ total: 4, story: 4, beauty: 0 });
	});

	test('applies simulated join floors when a member first becomes active after a later cutoff', () => {
		const history = [
			round(1, ['Alice'], { Alice: 10 }, {}, { Alice: 10 }, {}),
			round(2, ['Alice'], { Alice: 5 }, {}, { Alice: 15 }, {}),
			round(3, ['Alice', 'Bob'], { Bob: 2 }, {}, { Alice: 15, Bob: 12 }, {})
		];

		const breakdowns = sinceJoinedScoreBreakdowns(
			[entry('Alice', 15), entry('Bob', 12)],
			history,
			2
		);

		expect(breakdowns.get('Alice')).toEqual({ total: 5, story: 5, beauty: 0 });
		expect(breakdowns.get('Bob')).toEqual({ total: 7, story: 7, beauty: 0 });
	});

	test('uses snapshot offsets for beauty totals and story breakdowns', () => {
		const history = [
			round(1, ['Alice'], { Alice: 10 }, { Alice: 4 }, { Alice: 10 }, { Alice: 4 }),
			round(
				2,
				['Alice'],
				{ Observer: 3 },
				{ Observer: 1 },
				{ Alice: 10, Observer: 3 },
				{ Alice: 4, Observer: 1 }
			),
			round(
				3,
				['Alice', 'Observer'],
				{ Observer: 2 },
				{ Observer: 1 },
				{ Alice: 10, Observer: 5 },
				{ Alice: 4, Observer: 2 }
			)
		];

		const breakdowns = sinceJoinedScoreBreakdowns(
			[entry('Alice', 10, 4), entry('Observer', 5, 2)],
			history,
			1
		);

		expect(breakdowns.get('Alice')).toEqual({ total: 10, story: 6, beauty: 4 });
		expect(breakdowns.get('Observer')).toEqual({ total: 5, story: 3, beauty: 2 });
	});
});
