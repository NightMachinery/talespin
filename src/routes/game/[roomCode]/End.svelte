<script lang="ts">
	import {
		leaderboardViewMode,
		memberBeautyPoints,
		setLeaderboardViewModePreference
	} from '$lib/mostBeautiful';
	import {
		clueRatingEnabled,
		memberClueRatingAverage,
		memberClueRatingRounds
	} from '$lib/clueRating';
	import {
		firstActiveRoundForPlayer,
		leaderboardDigitWidths,
		leaderboardRoundHistory,
		resolveLeaderboardMode,
		rankEntriesByMode,
		scoreBreakdown,
		sinceJoinedScoreBreakdowns,
		type RankedLeaderboardEntry
	} from '$lib/leaderboard';
	import MigrateDeviceButton from '$lib/MigrateDeviceButton.svelte';
	import type { GameMode, LeaderboardViewMode, ObserverInfo, PlayerInfo } from '$lib/types';
	import MostBeautifulStatsPanel from './MostBeautifulStatsPanel.svelte';

	type CombinedScoreKey = 'total' | 'story' | 'beauty';
	type EndLeaderboardPlayerEntry = {
		name: string;
		breakdown: ReturnType<typeof scoreBreakdown>;
		clueStars: number | null;
	};
	type EndLeaderboardObserverEntry = {
		name: string;
		breakdown: ReturnType<typeof scoreBreakdown> | null;
		clueStars: number | null;
	};

	const COMBINED_SCORE_KEYS: CombinedScoreKey[] = ['total', 'story', 'beauty'];
	const COMBINED_SCORE_LABELS: Record<CombinedScoreKey, string> = {
		total: 'T',
		story: 'S',
		beauty: 'B'
	};

	export let players: { [key: string]: PlayerInfo } = {};
	export let observers: { [key: string]: ObserverInfo } = {};
	export let gameMode: GameMode = 'dixit_plus';
	export let beautyEnabled = false;
	export let name = '';
	export let roundNum = 0;
	export let storytellerPoolActive = false;
	export let storytellerPoolPlayers: string[] = [];

	let rankedPlayers: RankedLeaderboardEntry[] = [];
	let sortedObserverEntries: Array<{
		name: string;
		breakdown: ReturnType<typeof scoreBreakdown> | null;
		clueStars: number | null;
	}> = [];
	let activeEntries: {
		players: EndLeaderboardPlayerEntry[];
		observers: EndLeaderboardObserverEntry[];
	} = {
		players: [],
		observers: []
	};
	let digitWidths = { total: 1, story: 1, beauty: 1 };
	let showSinceJoined = false;
	let activeLeaderboardViewMode: LeaderboardViewMode = 'total';

	$: supportsBeautyModes = gameMode === 'dixit_plus' && beautyEnabled;
	$: supportsClueStarsMode = gameMode === 'dixit_plus' && $clueRatingEnabled;
	$: storytellerPoolPlayerSet = new Set(storytellerPoolPlayers);
	$: activeLeaderboardViewMode = resolveLeaderboardMode(
		$leaderboardViewMode,
		supportsBeautyModes,
		supportsClueStarsMode
	);
	$: if ($leaderboardViewMode !== activeLeaderboardViewMode) {
		setLeaderboardViewModePreference(activeLeaderboardViewMode);
	}
	$: viewerJoinedRound = firstActiveRoundForPlayer($leaderboardRoundHistory, name);
	$: canShowSinceJoined = viewerJoinedRound !== null;
	$: if (!canShowSinceJoined || activeLeaderboardViewMode === 'clue_stars') {
		showSinceJoined = false;
	}
	$: {
		$leaderboardRoundHistory;
		players;
		observers;
		$memberBeautyPoints;
		activeEntries =
			showSinceJoined && viewerJoinedRound !== null ? sinceJoinedEntries() : fullGameEntries();
	}
	$: rankedPlayers = rankEntriesByMode(activeEntries.players, activeLeaderboardViewMode);
	$: sortedObserverEntries = activeEntries.observers;
	$: digitWidths = leaderboardDigitWidths([
		...rankedPlayers,
		...sortedObserverEntries.flatMap((entry) =>
			entry.breakdown ? [{ breakdown: entry.breakdown }] : []
		)
	]);

	function allKnownNames() {
		return Array.from(
			new Set([
				...Object.keys(players),
				...Object.keys(observers),
				...$leaderboardRoundHistory.flatMap((round) => Object.keys(round.total_after_round))
			])
		).sort((a, b) => a.localeCompare(b));
	}

	function fullGameEntries() {
		const names = allKnownNames();
		const playerEntries = Object.entries(players).map(([playerName, info]) => ({
			name: playerName,
			breakdown: scoreBreakdown(info.points, $memberBeautyPoints[playerName] ?? 0),
			clueStars: clueStarsForPlayer(playerName)
		}));
		const observerEntries = Object.entries(observers)
			.sort(([a], [b]) => a.localeCompare(b))
			.map(([observerName, info]) => ({
				name: observerName,
				breakdown:
					info.points === null
						? null
						: scoreBreakdown(info.points, $memberBeautyPoints[observerName] ?? 0),
				clueStars: clueStarsForPlayer(observerName)
			}));
		for (const knownName of names) {
			if (!(knownName in players) && !(knownName in observers)) {
				playerEntries.push({
					name: knownName,
					breakdown: scoreBreakdown(0, 0),
					clueStars: clueStarsForPlayer(knownName)
				});
			}
		}
		return {
			players: playerEntries,
			observers: observerEntries
		};
	}

	function sinceJoinedEntries() {
		const currentScoreEntries = [
			...Object.entries(players).map(([playerName, info]) => ({
				name: playerName,
				total: info.points,
				beauty: $memberBeautyPoints[playerName] ?? 0,
				isPlayer: true,
				hasScore: true
			})),
			...Object.entries(observers).map(([observerName, info]) => ({
				name: observerName,
				total: info.points ?? 0,
				beauty: $memberBeautyPoints[observerName] ?? 0,
				isPlayer: false,
				hasScore: info.points !== null
			}))
		];
		const breakdowns = sinceJoinedScoreBreakdowns(currentScoreEntries, $leaderboardRoundHistory);
		const playerEntries = Object.keys(players)
			.sort((a, b) => a.localeCompare(b))
			.map((playerName) => ({
				name: playerName,
				breakdown: breakdowns.get(playerName) ?? scoreBreakdown(0, 0),
				clueStars: clueStarsForPlayer(playerName)
			}));
		const observerEntries = Object.entries(observers)
			.sort(([a], [b]) => a.localeCompare(b))
			.map(([observerName, info]) => ({
				name: observerName,
				breakdown:
					info.points === null ? null : breakdowns.get(observerName) ?? scoreBreakdown(0, 0),
				clueStars: clueStarsForPlayer(observerName)
			}));
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

	function isStorytellerPoolPlayer(playerName: string) {
		return storytellerPoolActive && storytellerPoolPlayerSet.has(playerName);
	}

	function clueStarsForPlayer(playerName: string) {
		if (($memberClueRatingRounds[playerName] ?? 0) < 1) return null;
		return $memberClueRatingAverage[playerName] ?? null;
	}

	function formatClueStars(value: number | null) {
		return value === null ? 'NA' : value.toFixed(1);
	}
</script>

<div class="px-3">
	<div class="mx-auto flex max-w-3xl flex-col items-center gap-4">
		<h1 class="text-center text-4xl">Game Over!</h1>

		<div class="card light w-full max-w-2xl p-4 text-center">
			<div class="mb-4 space-y-3">
				{#if supportsBeautyModes || supportsClueStarsMode}
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
							{#if supportsBeautyModes}
								<option value="beauty_only">Beauty Only</option>
								<option value="story_only">Story Only</option>
								<option value="combined">Combined</option>
							{/if}
							{#if supportsClueStarsMode}
								<option value="clue_stars">Clue Stars</option>
							{/if}
						</select>
					</label>
				{/if}
				{#if canShowSinceJoined && viewerJoinedRound !== null && activeLeaderboardViewMode !== 'clue_stars'}
					<label class="mx-auto flex max-w-xs items-start gap-3 text-left text-sm">
						<input
							type="checkbox"
							class="mt-0.5 h-4 w-4 cursor-pointer accent-primary-500"
							bind:checked={showSinceJoined}
						/>
						<span>Show each player’s score since they joined.</span>
					</label>
				{/if}
				<p class="text-sm opacity-80">Rounds played: {roundNum}</p>
				<div class="flex justify-center">
					<MigrateDeviceButton />
				</div>
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
							{#if isStorytellerPoolPlayer(entry.name)}
								<span class="storyteller-pool-badge" title="In storyteller pool">✦</span>
							{/if}
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
							{:else if activeLeaderboardViewMode === 'clue_stars'}
								{formatClueStars(entry.clueStars)}
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
								<div class="flex-auto">
									{observerEntry.name}
									{#if isStorytellerPoolPlayer(observerEntry.name)}
										<span class="storyteller-pool-badge" title="In storyteller pool">✦</span>
									{/if}
								</div>
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
									{:else if activeLeaderboardViewMode === 'clue_stars'}
										{formatClueStars(observerEntry.clueStars ?? null)}
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

		{#if gameMode === 'dixit_plus' && beautyEnabled}
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

	.storyteller-pool-badge {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		margin-left: 0.35rem;
		padding: 0 0.35rem;
		border: 1px solid rgb(255 255 255 / 0.28);
		border-radius: 9999px;
		font-size: 0.72rem;
		line-height: 1.2;
		opacity: 0.78;
	}
</style>
