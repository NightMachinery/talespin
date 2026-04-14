<script lang="ts">
	import {
		leaderboardViewMode,
		memberBeautyPoints,
		setLeaderboardViewModePreference
	} from '$lib/mostBeautiful';
	import {
		leaderboardDigitWidths,
		rankEntriesByMode,
		scoreBreakdown,
		type RankedLeaderboardEntry
	} from '$lib/leaderboard';
	import type {
		DixitEndRoundHistoryEntry,
		GameMode,
		LeaderboardViewMode,
		ObserverInfo,
		PlayerInfo
	} from '$lib/types';
	import MostBeautifulStatsPanel from './MostBeautifulStatsPanel.svelte';

	type CombinedScoreKey = 'total' | 'story' | 'beauty';

	const COMBINED_SCORE_KEYS: CombinedScoreKey[] = ['total', 'story', 'beauty'];
	const COMBINED_SCORE_LABELS: Record<CombinedScoreKey, string> = {
		total: 'T',
		story: 'S',
		beauty: 'B'
	};

	export let players: { [key: string]: PlayerInfo } = {};
	export let observers: { [key: string]: ObserverInfo } = {};
	export let gameMode: GameMode = 'dixit_plus';
	export let name = '';
	export let roundNum = 0;
	export let dixitEndRoundHistory: DixitEndRoundHistoryEntry[] = [];

	let rankedPlayers: RankedLeaderboardEntry[] = [];
	let sortedObserverEntries: Array<{
		name: string;
		breakdown: ReturnType<typeof scoreBreakdown> | null;
	}> = [];
	let digitWidths = { total: 1, story: 1, beauty: 1 };
	let showSinceJoined = false;

	$: supportsBeautyModes = gameMode === 'dixit_plus';
	$: activeLeaderboardViewMode = supportsBeautyModes ? $leaderboardViewMode : 'total';
	$: viewerJoinedRound = firstActiveRoundForPlayer(name);
	$: canShowSinceJoined = supportsBeautyModes && viewerJoinedRound !== null;
	$: if (!canShowSinceJoined) {
		showSinceJoined = false;
	}
	$: activeEntries =
		showSinceJoined && viewerJoinedRound !== null
			? cutoffEntries(viewerJoinedRound)
			: fullGameEntries();
	$: rankedPlayers = rankEntriesByMode(activeEntries.players, activeLeaderboardViewMode);
	$: sortedObserverEntries = activeEntries.observers;
	$: digitWidths = leaderboardDigitWidths([
		...rankedPlayers,
		...sortedObserverEntries.flatMap((entry) =>
			entry.breakdown ? [{ breakdown: entry.breakdown }] : []
		)
	]);

	function firstActiveRoundForPlayer(playerName: string) {
		for (const round of dixitEndRoundHistory) {
			if (round.active_players.includes(playerName)) {
				return round.round_num;
			}
		}
		return null;
	}

	function allKnownNames() {
		return Array.from(
			new Set([
				...Object.keys(players),
				...Object.keys(observers),
				...dixitEndRoundHistory.flatMap((round) => Object.keys(round.total_after_round))
			])
		).sort((a, b) => a.localeCompare(b));
	}

	function fullGameEntries() {
		const names = allKnownNames();
		const playerEntries = Object.entries(players).map(([playerName, info]) => ({
			name: playerName,
			breakdown: scoreBreakdown(info.points, $memberBeautyPoints[playerName] ?? 0)
		}));
		const observerEntries = Object.entries(observers)
			.sort(([a], [b]) => a.localeCompare(b))
			.map(([observerName, info]) => ({
				name: observerName,
				breakdown:
					info.points === null
						? null
						: scoreBreakdown(info.points, $memberBeautyPoints[observerName] ?? 0)
			}));
		for (const knownName of names) {
			if (!(knownName in players) && !(knownName in observers)) {
				playerEntries.push({
					name: knownName,
					breakdown: scoreBreakdown(0, 0)
				});
			}
		}
		return {
			players: playerEntries,
			observers: observerEntries
		};
	}

	function cutoffEntries(cutoffRound: number) {
		const totals = new Map<string, { total: number; beauty: number }>();
		for (const round of dixitEndRoundHistory) {
			if (round.round_num < cutoffRound) continue;
			for (const [playerName, delta] of Object.entries(round.story_deltas)) {
				const current = totals.get(playerName) ?? { total: 0, beauty: 0 };
				current.total += delta;
				totals.set(playerName, current);
			}
			for (const [playerName, delta] of Object.entries(round.beauty_deltas)) {
				const current = totals.get(playerName) ?? { total: 0, beauty: 0 };
				current.total += delta;
				current.beauty += delta;
				totals.set(playerName, current);
			}
		}
		const names = allKnownNames();
		const playerEntries = names
			.filter((playerName) => playerName in players)
			.map((playerName) => {
				const current = totals.get(playerName) ?? { total: 0, beauty: 0 };
				return {
					name: playerName,
					breakdown: scoreBreakdown(current.total, current.beauty)
				};
			});
		const observerEntries = names
			.filter((playerName) => playerName in observers)
			.map((playerName) => {
				const current = totals.get(playerName) ?? { total: 0, beauty: 0 };
				return {
					name: playerName,
					breakdown: scoreBreakdown(current.total, current.beauty)
				};
			});
		return {
			players: playerEntries,
			observers: observerEntries
		};
	}

	function combinedHeaderLabel(key: CombinedScoreKey) {
		return COMBINED_SCORE_LABELS[key];
	}

	function combinedColumns(breakdown: ReturnType<typeof scoreBreakdown>) {
		return [
			{ key: 'total' as const, value: breakdown.total },
			{ key: 'story' as const, value: breakdown.story },
			{ key: 'beauty' as const, value: breakdown.beauty }
		];
	}

	function handleViewModeChange(event: Event) {
		const select = event.currentTarget as HTMLSelectElement;
		setLeaderboardViewModePreference(select.value as LeaderboardViewMode);
	}

	function widthStyle(digits: number) {
		return `min-width: ${digits}ch;`;
	}
</script>

<div class="px-3">
	<div class="mx-auto flex max-w-3xl flex-col items-center gap-4">
		<h1 class="text-center text-4xl">Game Over!</h1>

		<div class="card light w-full max-w-2xl p-4 text-center">
			<div class="mb-4 space-y-3">
				{#if supportsBeautyModes}
					<label class="mx-auto block max-w-xs text-left text-sm">
						<span class="mb-1 block text-xs font-semibold uppercase tracking-wide opacity-70">
							Leaderboard view
						</span>
						<select
							class="w-full rounded border px-3 py-2 text-gray-700 shadow"
							value={$leaderboardViewMode}
							on:change={handleViewModeChange}
						>
							<option value="total">Total</option>
							<option value="beauty_only">Beauty Only</option>
							<option value="story_only">Story Only</option>
							<option value="combined">Combined</option>
						</select>
					</label>
					{#if canShowSinceJoined && viewerJoinedRound !== null}
						<label class="mx-auto flex max-w-xs items-start gap-3 text-left text-sm">
							<input
								type="checkbox"
								class="mt-0.5 h-4 w-4 cursor-pointer accent-primary-500"
								bind:checked={showSinceJoined}
							/>
							<span
								>Show leaderboard starting from round {viewerJoinedRound}, when you first joined.</span
							>
						</label>
					{/if}
				{/if}
				<p class="text-sm opacity-80">Rounds played: {roundNum}</p>
			</div>

			{#if activeLeaderboardViewMode === 'combined'}
				<div
					class="mb-2 flex justify-end pr-1 text-[11px] font-semibold uppercase tracking-wide opacity-60"
				>
					<div class="combined-score-header">
						{#each COMBINED_SCORE_KEYS as key}
							<div class="combined-score-group">
								<span class="score-column" style={widthStyle(digitWidths[key])}
									>{combinedHeaderLabel(key)}</span
								>
							</div>
						{/each}
					</div>
				</div>
			{/if}

			<div class="space-y-1 text-xl">
				{#each rankedPlayers as entry}
					<div class="flex items-center justify-between gap-3">
						<div class="flex-auto text-left">
							{entry.rank}.
							<span class={`${entry.isTopScore ? 'boujee-text' : ''}`}>{entry.name}</span>
						</div>
						<div class="shrink-0 text-right font-mono tabular-nums">
							{#if activeLeaderboardViewMode === 'combined'}
								<div class="combined-score">
									{#each combinedColumns(entry.breakdown) as column}
										<div class="combined-score-group">
											<span class="score-column" style={widthStyle(digitWidths[column.key])}
												>{column.value}</span
											>
										</div>
									{/each}
								</div>
							{:else if activeLeaderboardViewMode === 'beauty_only'}
								{entry.breakdown.beauty}
							{:else if activeLeaderboardViewMode === 'story_only'}
								{entry.breakdown.story}
							{:else}
								{entry.breakdown.total}
							{/if}
						</div>
					</div>
				{/each}
			</div>

			{#if sortedObserverEntries.length > 0}
				<div class="mt-4 border-t border-white/15 pt-3 text-left">
					<h2 class="mb-2 text-lg font-semibold">Observers</h2>
					<div class="space-y-1 text-xl opacity-85">
						{#each sortedObserverEntries as observerEntry}
							<div class="flex items-center justify-between gap-3">
								<div class="flex-auto">{observerEntry.name}</div>
								<div class="shrink-0 text-right font-mono tabular-nums">
									{#if observerEntry.breakdown === null}
										NA
									{:else if activeLeaderboardViewMode === 'combined'}
										<div class="combined-score">
											{#each combinedColumns(observerEntry.breakdown) as column}
												<div class="combined-score-group">
													<span class="score-column" style={widthStyle(digitWidths[column.key])}
														>{column.value}</span
													>
												</div>
											{/each}
										</div>
									{:else if activeLeaderboardViewMode === 'beauty_only'}
										{observerEntry.breakdown.beauty}
									{:else if activeLeaderboardViewMode === 'story_only'}
										{observerEntry.breakdown.story}
									{:else}
										{observerEntry.breakdown.total}
									{/if}
								</div>
							</div>
						{/each}
					</div>
				</div>
			{/if}
		</div>

		{#if gameMode === 'dixit_plus'}
			<div class="w-full max-w-2xl">
				<MostBeautifulStatsPanel title="Most Beautiful ranking" />
			</div>
		{/if}
	</div>
</div>

<style>
	.score-column {
		display: inline-block;
		text-align: right;
	}

	.combined-score,
	.combined-score-header {
		display: flex;
		align-items: baseline;
		justify-content: flex-end;
	}

	.combined-score-group {
		display: flex;
		align-items: baseline;
	}

	.combined-score-group + .combined-score-group {
		margin-left: 0.55rem;
		padding-left: 0.55rem;
		border-left: 1px solid rgb(255 255 255 / 0.18);
	}
</style>
