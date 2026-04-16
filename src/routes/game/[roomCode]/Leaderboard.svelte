<script lang="ts">
	import { onDestroy } from 'svelte';
	import type GameServer from '$lib/gameServer';
	import {
		beautyLeaderboardPointChange,
		leaderboardViewMode,
		memberBeautyPoints,
		roomLeaderboardViewModeDefault,
		setLeaderboardViewModePreference,
		storytellerLeaderboardPointChange
	} from '$lib/mostBeautiful';
	import {
		leaderboardDigitWidths,
		leaderboardModeLabel,
		rankEntriesByMode,
		scoreBreakdown,
		type RankedLeaderboardEntry
	} from '$lib/leaderboard';
	import type {
		GameMode,
		LeaderboardViewMode,
		ObserverInfo,
		PlayerInfo,
		WinCondition
	} from '$lib/types';
	import { OFFLINE_STATUS_LABEL } from '$lib/presence';
	import { formatWinCondition } from '$lib/winCondition';

	type CombinedScoreKey = 'total' | 'story' | 'beauty';

	const COMBINED_SCORE_KEYS: CombinedScoreKey[] = ['total', 'story', 'beauty'];
	const COMBINED_SCORE_LABELS: Record<CombinedScoreKey, string> = {
		total: 'T',
		story: 'S',
		beauty: 'B'
	};

	export let players: { [key: string]: PlayerInfo } = {};
	export let observers: { [key: string]: ObserverInfo } = {};
	export let name = '';
	export let moderators: string[] = [];
	export let gameServer: GameServer;
	export let stage = '';
	export let activePlayer = '';
	export let storytellerPoolActive = false;
	export let storytellerPoolPlayers: string[] = [];
	export let pointChange: { [key: string]: number } = {};
	export let roundNum: number;
	export let cardsRemaining = 0;
	export let deckRefillFlashToken = 0;
	export let darkPlayer = '';
	export let gameMode: GameMode = 'dixit_plus';
	export let beautyEnabled = false;
	export let leaderboardPointChangeStageOverride = '';
	export let leaderboardPointChangeOverride: { [key: string]: number } | null = null;
	export let leaderboardStoryPointChangeOverride: { [key: string]: number } | null = null;
	export let leaderboardBeautyPointChangeOverride: { [key: string]: number } | null = null;
	export let winCondition: WinCondition = {
		mode: 'points',
		target_points: 10
	};

	let rankedPlayers: RankedLeaderboardEntry[] = [];
	let winConditionLabel = '';
	let previousFlashToken = 0;
	let cardsRemainingFlashing = false;
	let cardsRemainingFlashTimeout: ReturnType<typeof setTimeout> | undefined = undefined;
	let sortedObserverEntries: [string, ObserverInfo][] = [];
	let adjustedObserverEntries: Array<{
		name: string;
		info: ObserverInfo;
		breakdown: ReturnType<typeof scoreBreakdown> | null;
		displayPointChange: number | null;
	}> = [];
	let digitWidths = { total: 1, story: 1, beauty: 1 };
	let combinedDeltaWidths = { total: 2, story: 2, beauty: 2 };

	$: isModerator = new Set(moderators).has(name);
	$: storytellerPoolPlayerSet = new Set(storytellerPoolPlayers);
	$: supportsBeautyModes = gameMode === 'dixit_plus' && beautyEnabled;
	$: activeLeaderboardViewMode = supportsBeautyModes ? $leaderboardViewMode : 'total';
	$: pointChangeStage = leaderboardPointChangeStageOverride || stage;
	$: effectivePointChange = leaderboardPointChangeOverride ?? pointChange;
	$: effectiveStorytellerPointChange =
		leaderboardStoryPointChangeOverride ?? $storytellerLeaderboardPointChange;
	$: effectiveBeautyPointChange =
		leaderboardBeautyPointChangeOverride ?? $beautyLeaderboardPointChange;
	$: {
		rankedPlayers = rankEntriesByMode(
			Object.entries(players).map(([entryName, info]) => ({
				name: entryName,
				breakdown: scoreBreakdown(info.points, beautyPointsForPlayer(entryName))
			})),
			activeLeaderboardViewMode
		);
	}
	$: sortedObserverEntries = Object.entries(observers).sort(([a], [b]) => a.localeCompare(b));
	$: adjustedObserverEntries = sortedObserverEntries.map(([observerName, info]) => ({
		name: observerName,
		info,
		breakdown:
			info.points === null
				? null
				: scoreBreakdown(info.points, beautyPointsForPlayer(observerName)),
		displayPointChange: displayedPointChangeForPlayer(observerName)
	}));
	$: digitWidths = leaderboardDigitWidths([
		...rankedPlayers,
		...adjustedObserverEntries.flatMap((entry) =>
			entry.breakdown ? [{ breakdown: entry.breakdown }] : []
		)
	]);
	$: combinedDeltaWidths = {
		total: combinedDeltaWidth('total'),
		story: combinedDeltaWidth('story'),
		beauty: combinedDeltaWidth('beauty')
	};

	$: winConditionLabel = formatWinCondition(winCondition);

	$: if (deckRefillFlashToken > previousFlashToken) {
		previousFlashToken = deckRefillFlashToken;
		cardsRemainingFlashing = true;
		if (cardsRemainingFlashTimeout) {
			clearTimeout(cardsRemainingFlashTimeout);
		}
		cardsRemainingFlashTimeout = setTimeout(() => {
			cardsRemainingFlashing = false;
		}, 900);
	}

	onDestroy(() => {
		if (cardsRemainingFlashTimeout) {
			clearTimeout(cardsRemainingFlashTimeout);
		}
	});

	function shouldShowReadyIndicator(playerName: string) {
		if (
			stage === 'Joining' ||
			stage === 'StellaAssociate' ||
			stage === 'BeautyVoting' ||
			stage === 'Results' ||
			stage === 'BeautyResults' ||
			stage === 'StellaResults'
		) {
			return true;
		}
		if ((stage === 'PlayersChoose' || stage === 'Voting') && playerName !== activePlayer) {
			return true;
		}
		return false;
	}

	function shouldShowPointChange() {
		return (
			pointChangeStage === 'Results' ||
			pointChangeStage === 'BeautyResults' ||
			pointChangeStage === 'StellaReveal' ||
			pointChangeStage === 'StellaResults'
		);
	}

	function shouldShowCombinedDeltaColumns() {
		return activeLeaderboardViewMode === 'combined' && shouldShowPointChange();
	}

	function beautyPointsForPlayer(playerName: string) {
		return $memberBeautyPoints[playerName] ?? 0;
	}

	function displayedPointChangeForPlayer(playerName: string) {
		if (!supportsBeautyModes) {
			return effectivePointChange[playerName] ?? 0;
		}
		switch (activeLeaderboardViewMode) {
			case 'story_only':
				if (pointChangeStage === 'BeautyResults') {
					return 0;
				}
				return effectiveStorytellerPointChange[playerName] ?? 0;
			case 'beauty_only':
				return effectiveBeautyPointChange[playerName] ?? 0;
			case 'combined':
				return null;
			case 'total':
			default:
				return effectivePointChange[playerName] ?? 0;
		}
	}

	function combinedPointChangeForPlayer(playerName: string) {
		if (!supportsBeautyModes || !shouldShowPointChange()) {
			return null;
		}
		return {
			total: effectivePointChange[playerName] ?? 0,
			story:
				pointChangeStage === 'BeautyResults' ? 0 : effectiveStorytellerPointChange[playerName] ?? 0,
			beauty: effectiveBeautyPointChange[playerName] ?? 0
		};
	}

	function combinedColumns(
		playerName: string,
		breakdown: ReturnType<typeof scoreBreakdown>
	): Array<{ key: CombinedScoreKey; value: number; delta: number | null }> {
		const deltas = combinedPointChangeForPlayer(playerName);
		return [
			{
				key: 'total',
				value: breakdown.total,
				delta: deltas?.total ?? null
			},
			{
				key: 'story',
				value: breakdown.story,
				delta: deltas?.story ?? null
			},
			{
				key: 'beauty',
				value: breakdown.beauty,
				delta: deltas?.beauty ?? null
			}
		];
	}

	function combinedHeaderLabel(key: CombinedScoreKey) {
		return COMBINED_SCORE_LABELS[key];
	}

	function combinedDeltaWidth(key: CombinedScoreKey) {
		if (!shouldShowCombinedDeltaColumns()) {
			return 2;
		}

		const values = [
			...rankedPlayers.map((entry) => combinedPointChangeForPlayer(entry.name)?.[key] ?? null),
			...adjustedObserverEntries.map((entry) =>
				entry.breakdown === null ? null : combinedPointChangeForPlayer(entry.name)?.[key] ?? null
			)
		].filter((value): value is number => value !== null);

		return Math.max(2, ...values.map((value) => formatSignedDelta(value).length));
	}

	function digitWidthForKey(key: CombinedScoreKey) {
		return widthStyle(digitWidths[key]);
	}

	function combinedDeltaWidthStyle(key: CombinedScoreKey) {
		return widthStyle(combinedDeltaWidths[key]);
	}

	function formatSignedDelta(delta: number) {
		return `${delta >= 0 ? '+' : ''}${delta}`;
	}

	function handleViewModeChange(event: Event) {
		const select = event.currentTarget as HTMLSelectElement;
		setLeaderboardViewModePreference(select.value as LeaderboardViewMode);
	}

	function pushCurrentViewToRoom() {
		gameServer.setLeaderboardViewModeDefault($leaderboardViewMode);
	}

	function isStorytellerPoolPlayer(playerName: string) {
		return storytellerPoolActive && storytellerPoolPlayerSet.has(playerName);
	}

	function widthStyle(digits: number) {
		return `min-width: ${digits}ch;`;
	}
</script>

<div class="w-full">
	<div class="card light p-4">
		<h2 class="text-xl">Round {roundNum}</h2>
		{#if supportsBeautyModes}
			<div class="mt-3 space-y-2 text-sm">
				<label class="block">
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
				{#if isModerator && stage !== 'End'}
					<div class="space-y-1">
						<button class="btn variant-soft w-full" on:click={pushCurrentViewToRoom}>
							Force-push “{leaderboardModeLabel($leaderboardViewMode)}” view to room
						</button>
						<p class="text-xs opacity-70">
							Room default: {leaderboardModeLabel($roomLeaderboardViewModeDefault)}
						</p>
					</div>
				{/if}
			</div>
		{/if}
		<div class="mt-3">
			{#if activeLeaderboardViewMode === 'combined'}
				<div
					class="mb-2 flex justify-end pr-1 text-[11px] font-semibold uppercase tracking-wide opacity-60"
				>
					<div class="combined-score-header">
						{#each COMBINED_SCORE_KEYS as key}
							<div class="combined-score-group">
								<span class="score-column" style={digitWidthForKey(key)}
									>{combinedHeaderLabel(key)}</span
								>
								{#if shouldShowCombinedDeltaColumns()}
									<span
										class="combined-delta combined-delta-placeholder"
										style={combinedDeltaWidthStyle(key)}
										aria-hidden="true"
									></span>
								{/if}
							</div>
						{/each}
					</div>
				</div>
			{/if}
			<div>
				{#each rankedPlayers as entry}
					<div class="flex items-center justify-between gap-2 py-0.5">
						<div class="flex-auto">
							{entry.rank}.
							<span
								class={`${entry.name === activePlayer ? 'boujee-text' : ''} ${!players[entry.name].connected ? 'opacity-50 grayscale' : ''}`}
							>
								{entry.name}
							</span>
							{#if isStorytellerPoolPlayer(entry.name)}
								<span class="storyteller-pool-badge" title="In storyteller pool">✦</span>
							{/if}
							{#if darkPlayer !== '' && entry.name === darkPlayer}
								<span title="In the Dark">🌑</span>
							{/if}
							{#if !players[entry.name].connected}
								<span class="text-error-500">({OFFLINE_STATUS_LABEL})</span>
							{/if}
							{#if shouldShowReadyIndicator(entry.name)}
								{#if players[entry.name].ready}
									<span>🟢</span>
								{:else}
									<span>🔴</span>
								{/if}
							{/if}
						</div>
						<div class="shrink-0 text-right font-mono tabular-nums">
							{#if activeLeaderboardViewMode !== 'combined' && shouldShowPointChange() && displayedPointChangeForPlayer(entry.name) !== 0}
								<span class="mr-2 opacity-50"
									>({formatSignedDelta(displayedPointChangeForPlayer(entry.name) ?? 0)})</span
								>
							{/if}
							{#if activeLeaderboardViewMode === 'combined'}
								<div class="combined-score">
									{#each combinedColumns(entry.name, entry.breakdown) as column}
										<div class="combined-score-group">
											<span class="score-column" style={digitWidthForKey(column.key)}
												>{column.value}</span
											>
											{#if shouldShowCombinedDeltaColumns()}
												<span
													class={`combined-delta ${column.delta === 0 ? 'opacity-45' : ''}`}
													style={combinedDeltaWidthStyle(column.key)}
												>
													{formatSignedDelta(column.delta ?? 0)}
												</span>
											{/if}
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
		</div>
		<br />
		<p>{winConditionLabel}</p>
		<p
			class={`mt-1 inline-block rounded px-2 py-0.5 ${cardsRemainingFlashing ? 'cards-refilled-flash' : ''}`}
		>
			Cards left: {cardsRemaining}
		</p>
		{#if Object.keys(observers).length > 0}
			<div class="mt-3 text-sm opacity-80">
				<p class="font-semibold">Observers</p>
				{#if activeLeaderboardViewMode === 'combined'}
					<div
						class="mb-2 mt-1 flex justify-end pr-1 text-[11px] font-semibold uppercase tracking-wide opacity-60"
					>
						<div class="combined-score-header">
							{#each COMBINED_SCORE_KEYS as key}
								<div class="combined-score-group">
									<span class="score-column" style={digitWidthForKey(key)}
										>{combinedHeaderLabel(key)}</span
									>
									{#if shouldShowCombinedDeltaColumns()}
										<span
											class="combined-delta combined-delta-placeholder"
											style={combinedDeltaWidthStyle(key)}
											aria-hidden="true"
										></span>
									{/if}
								</div>
							{/each}
						</div>
					</div>
				{/if}
				<div class="mt-1 space-y-1">
					{#each adjustedObserverEntries as observerEntry}
						<div
							class={`flex items-center justify-between gap-2 ${!observerEntry.info.connected ? 'opacity-50' : ''}`}
						>
							<div class="min-w-0 flex-auto break-words">
								{observerEntry.name}
								{#if isStorytellerPoolPlayer(observerEntry.name)}
									<span class="storyteller-pool-badge" title="In storyteller pool">✦</span>
								{/if}
								{#if observerEntry.info.join_requested || observerEntry.info.auto_join_on_next_round}
									<span class="opacity-70"> (joining next round)</span>
								{/if}
								{#if !observerEntry.info.connected}
									<span class="opacity-70"> ({OFFLINE_STATUS_LABEL})</span>
								{/if}
							</div>
							<div class="shrink-0 text-right font-mono tabular-nums">
								{#if activeLeaderboardViewMode !== 'combined' && shouldShowPointChange() && observerEntry.displayPointChange !== 0}
									<span class="mr-2 opacity-50"
										>({formatSignedDelta(observerEntry.displayPointChange ?? 0)})</span
									>
								{/if}
								{#if observerEntry.breakdown === null}
									NA
								{:else if activeLeaderboardViewMode === 'combined'}
									<div class="combined-score">
										{#each combinedColumns(observerEntry.name, observerEntry.breakdown) as column}
											<div class="combined-score-group">
												<span class="score-column" style={digitWidthForKey(column.key)}
													>{column.value}</span
												>
												{#if shouldShowCombinedDeltaColumns()}
													<span
														class={`combined-delta ${column.delta === 0 ? 'opacity-45' : ''}`}
														style={combinedDeltaWidthStyle(column.key)}
													>
														{formatSignedDelta(column.delta ?? 0)}
													</span>
												{/if}
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
</div>

<style>
	.cards-refilled-flash {
		animation: cards-refilled-pulse 0.9s ease-out;
		background: rgba(255, 255, 255, 0.2);
	}

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
		gap: 0.18rem;
	}

	.combined-score-group + .combined-score-group {
		margin-left: 0.55rem;
		padding-left: 0.55rem;
		border-left: 1px solid rgb(255 255 255 / 0.18);
	}

	.combined-delta {
		display: inline-block;
		text-align: left;
		font-size: 0.72rem;
		opacity: 0.75;
	}

	.combined-delta-placeholder {
		visibility: hidden;
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

	@keyframes cards-refilled-pulse {
		0% {
			background: rgba(74, 222, 128, 0.65);
			transform: scale(1.02);
		}
		100% {
			background: rgba(255, 255, 255, 0.2);
			transform: scale(1);
		}
	}
</style>
