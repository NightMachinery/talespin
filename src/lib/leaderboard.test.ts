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
	test('uses each players own first active round instead of the viewers round', () => {
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
			history
		);

		expect(breakdowns.get('Alice')).toEqual({ total: 16, story: 16, beauty: 0 });
		expect(breakdowns.get('Bob')).toEqual({ total: 5, story: 5, beauty: 0 });
		expect(breakdowns.get('Carol')).toEqual({ total: 4, story: 4, beauty: 0 });
	});

	test('keeps scored observers without active history on their raw scores', () => {
		const history = [
			round(1, ['Alice'], { Alice: 10 }, {}, { Alice: 10 }, {}),
			round(2, ['Alice'], { Observer: 4 }, {}, { Alice: 10, Observer: 4 }, {})
		];

		const breakdowns = sinceJoinedScoreBreakdowns(
			[entry('Alice', 10), entry('Observer', 4, 0, false)],
			history
		);

		expect(breakdowns.get('Observer')).toEqual({ total: 4, story: 4, beauty: 0 });
	});

	test('excludes automatic join-floor points for later active players', () => {
		const history = [
			round(1, ['Alice'], { Alice: 10 }, {}, { Alice: 10 }, {}),
			round(2, ['Alice'], { Alice: 5 }, {}, { Alice: 15 }, {}),
			round(3, ['Alice', 'Bob'], { Bob: 2 }, {}, { Alice: 15, Bob: 12 }, {})
		];

		const breakdowns = sinceJoinedScoreBreakdowns([entry('Alice', 15), entry('Bob', 12)], history);

		expect(breakdowns.get('Alice')).toEqual({ total: 15, story: 15, beauty: 0 });
		expect(breakdowns.get('Bob')).toEqual({ total: 2, story: 2, beauty: 0 });
	});

	test('uses personal snapshot offsets for beauty totals and story breakdowns', () => {
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
			history
		);

		expect(breakdowns.get('Alice')).toEqual({ total: 10, story: 6, beauty: 4 });
		expect(breakdowns.get('Bob')).toEqual({ total: 7, story: 3, beauty: 4 });
	});

	test('shows zero for a live active player before their first recorded round', () => {
		const history = [round(1, ['Alice'], { Alice: 10 }, {}, { Alice: 10 }, {})];

		const breakdowns = sinceJoinedScoreBreakdowns([entry('Alice', 10), entry('Bob', 10)], history);

		expect(breakdowns.get('Bob')).toEqual({ total: 0, story: 0, beauty: 0 });
	});
});
