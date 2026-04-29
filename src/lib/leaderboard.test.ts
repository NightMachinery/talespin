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
			round(2, ['Alice'], { Alice: 5 }, {}, { Alice: 15 }, {}),
			round(3, ['Alice', 'Bob'], { Alice: 1, Bob: 2 }, {}, { Alice: 16, Bob: 17 }, {}),
			round(
				4,
				['Alice', 'Bob', 'Carol'],
				{ Bob: 3, Carol: 4 },
				{},
				{ Alice: 16, Bob: 20, Carol: 20 },
				{}
			)
		];

		const breakdowns = sinceJoinedScoreBreakdowns(
			[entry('Alice', 16), entry('Bob', 20), entry('Carol', 20)],
			history,
			1
		);

		expect(breakdowns.get('Alice')).toEqual({ total: 16, story: 16, beauty: 0 });
		expect(breakdowns.get('Bob')).toEqual({ total: 20, story: 20, beauty: 0 });
		expect(breakdowns.get('Carol')).toEqual({ total: 20, story: 20, beauty: 0 });
	});

	test('uses the viewer cutoff round and gives later active joiners the simulated minimum', () => {
		const history = [
			round(1, ['Existing'], { Existing: 10 }, {}, { Existing: 10 }, {}),
			round(2, ['Existing'], { Existing: 10 }, {}, { Existing: 20 }, {}),
			round(3, ['Existing', 'Bob'], { Existing: 5, Bob: 2 }, {}, { Existing: 25, Bob: 22 }, {}),
			round(4, ['Existing', 'Bob'], { Existing: 3, Bob: 4 }, {}, { Existing: 28, Bob: 26 }, {}),
			round(
				5,
				['Existing', 'Bob', 'Alice'],
				{ Existing: 1, Bob: 1, Alice: 3 },
				{},
				{ Existing: 29, Bob: 27, Alice: 29 },
				{}
			)
		];

		const breakdowns = sinceJoinedScoreBreakdowns(
			[entry('Existing', 29), entry('Bob', 27), entry('Alice', 29)],
			history,
			3
		);

		expect(breakdowns.get('Existing')).toEqual({ total: 9, story: 9, beauty: 0 });
		expect(breakdowns.get('Bob')).toEqual({ total: 7, story: 7, beauty: 0 });
		expect(breakdowns.get('Alice')).toEqual({ total: 9, story: 9, beauty: 0 });
	});

	test('gives live active joiners the simulated minimum before their first recorded round', () => {
		const history = [
			round(1, ['Existing'], { Existing: 10 }, {}, { Existing: 10 }, {}),
			round(2, ['Existing'], { Existing: 10 }, {}, { Existing: 20 }, {}),
			round(3, ['Existing', 'Bob'], { Existing: 5, Bob: 2 }, {}, { Existing: 25, Bob: 22 }, {}),
			round(4, ['Existing', 'Bob'], { Existing: 3, Bob: 4 }, {}, { Existing: 28, Bob: 26 }, {})
		];

		const breakdowns = sinceJoinedScoreBreakdowns(
			[entry('Existing', 28), entry('Bob', 26), entry('Alice', 26)],
			history,
			3
		);

		expect(breakdowns.get('Existing')).toEqual({ total: 8, story: 8, beauty: 0 });
		expect(breakdowns.get('Bob')).toEqual({ total: 6, story: 6, beauty: 0 });
		expect(breakdowns.get('Alice')).toEqual({ total: 6, story: 6, beauty: 0 });
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

	test('uses viewer-cutoff snapshot offsets for beauty totals and story breakdowns', () => {
		const history = [
			round(1, ['Alice'], { Alice: 10 }, { Alice: 4 }, { Alice: 10 }, { Alice: 4 }),
			round(
				2,
				['Alice', 'Bob'],
				{ Bob: 5 },
				{ Bob: 3 },
				{ Alice: 10, Bob: 15 },
				{ Alice: 4, Bob: 3 }
			),
			round(
				3,
				['Alice', 'Bob'],
				{ Bob: 2 },
				{ Bob: 1 },
				{ Alice: 10, Bob: 17 },
				{ Alice: 4, Bob: 4 }
			)
		];

		const breakdowns = sinceJoinedScoreBreakdowns(
			[entry('Alice', 10, 4), entry('Bob', 17, 4)],
			history,
			2
		);

		expect(breakdowns.get('Alice')).toEqual({ total: 0, story: 0, beauty: 0 });
		expect(breakdowns.get('Bob')).toEqual({ total: 7, story: 3, beauty: 4 });
	});
});
