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
		clueRatingEnabled,
		memberClueRatingAverage,
		memberClueRatingRounds,
		resultsPlayerClueRatings
	} from '$lib/clueRating';
	import {
		firstActiveRoundForPlayer,
		leaderboardDigitWidths,
		leaderboardModeLabel,
		leaderboardRoundHistory,
		resolveLeaderboardMode,
		rankEntriesByMode,
		scoreBreakdown,
		sinceJoinedScoreBreakdowns,
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
	export let leaderboardRoundClueRatingOverride: { [key: string]: number } | null = null;
	export let winCondition: WinCondition = {
		mode: 'points',
		target_points: 10
	};

	let rankedPlayers: RankedLeaderboardEntry[] = [];
	let activeLeaderboardViewMode: LeaderboardViewMode = 'total';
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
	let sinceJoinedBreakdowns = new Map<string, ReturnType<typeof scoreBreakdown>>();
	let showSinceJoined = false;
	let currentScoreEntriesForSinceJoined: ReturnType<typeof currentScoreEntries> = [];
	let digitWidths = { total: 1, story: 1, beauty: 1 };
	let combinedDeltaWidths = { total: 2, story: 2, beauty: 2 };

	$: isModerator = new Set(moderators).has(name);
	$: storytellerPoolPlayerSet = new Set(storytellerPoolPlayers);
	$: supportsBeautyModes = gameMode === 'dixit_plus' && beautyEnabled;
	$: supportsClueStarsMode = gameMode === 'dixit_plus' && $clueRatingEnabled;
	$: activeLeaderboardViewMode = resolveLeaderboardMode(
		$leaderboardViewMode,
		supportsBeautyModes,
		supportsClueStarsMode
	);
	$: if ($leaderboardViewMode !== activeLeaderboardViewMode) {
		setLeaderboardViewModePreference(activeLeaderboardViewMode);
	}
	$: pointChangeStage = leaderboardPointChangeStageOverride || stage;
	$: effectivePointChange = leaderboardPointChangeOverride ?? pointChange;
	$: effectiveStorytellerPointChange =
		leaderboardStoryPointChangeOverride ?? $storytellerLeaderboardPointChange;
	$: effectiveBeautyPointChange =
		leaderboardBeautyPointChangeOverride ?? $beautyLeaderboardPointChange;
	$: viewerJoinedRound = firstActiveRoundForPlayer($leaderboardRoundHistory, name);
	$: canShowSinceJoined = viewerJoinedRound !== null;
	$: if (!canShowSinceJoined || activeLeaderboardViewMode === 'clue_stars') {
		showSinceJoined = false;
	}
	$: currentScoreEntriesForSinceJoined = currentScoreEntries(
		players,
		observers,
		$memberBeautyPoints
	);
	$: sinceJoinedBreakdowns =
		showSinceJoined && viewerJoinedRound !== null
			? sinceJoinedScoreBreakdowns(
					currentScoreEntriesForSinceJoined,
					$leaderboardRoundHistory,
					viewerJoinedRound
				)
			: new Map();
	$: {
		sinceJoinedBreakdowns;
		rankedPlayers = rankEntriesByMode(
			Object.entries(players).map(([entryName, info]) => ({
				name: entryName,
				breakdown: displayedBreakdownForPlayer(entryName, info.points),
				clueStars: clueStarsForPlayer(entryName)
			})),
			activeLeaderboardViewMode
		);
	}
	$: sortedObserverEntries = Object.entries(observers).sort(([a], [b]) => a.localeCompare(b));
	$: {
		sinceJoinedBreakdowns;
		adjustedObserverEntries = sortedObserverEntries.map(([observerName, info]) => ({
			name: observerName,
			info,
			breakdown:
				info.points === null ? null : displayedBreakdownForPlayer(observerName, info.points),
			displayPointChange: displayedPointChangeForPlayer(observerName)
		}));
	}
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
			stage === 'ClueRating' ||
			stage === 'BeautyVoting' ||
			stage === 'Results' ||
			stage === 'BeautyResults' ||
			stage === 'StellaResults'
		) {
			return stage !== 'ClueRating' || playerName !== activePlayer;
		}
		if (
			(stage === 'PlayersChoose' || stage === 'Voting' || stage === 'ClueRating') &&
			playerName !== activePlayer
		) {
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

	function currentScoreEntries(
		currentPlayers: { [key: string]: PlayerInfo },
		currentObservers: { [key: string]: ObserverInfo },
		beautyPoints: Record<string, number>
	) {
		return [
			...Object.entries(currentPlayers).map(([playerName, info]) => ({
				name: playerName,
				total: info.points,
				beauty: beautyPoints[playerName] ?? 0,
				isPlayer: true,
				hasScore: true
			})),
			...Object.entries(currentObservers).map(([observerName, info]) => ({
				name: observerName,
				total: info.points ?? 0,
				beauty: beautyPoints[observerName] ?? 0,
				isPlayer: false,
				hasScore: info.points !== null
			}))
		];
	}

	function displayedBreakdownForPlayer(playerName: string, total: number) {
		return (
			sinceJoinedBreakdowns.get(playerName) ??
			scoreBreakdown(total, beautyPointsForPlayer(playerName))
		);
	}

	function displayedPointChangeForPlayer(playerName: string) {
		if (activeLeaderboardViewMode === 'clue_stars') {
			return 0;
		}
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

	function clueStarsForPlayer(playerName: string) {
		if (($memberClueRatingRounds[playerName] ?? 0) < 1) return null;
		return $memberClueRatingAverage[playerName] ?? null;
	}

	function formatClueStars(value: number | null) {
		return value === null ? 'NA' : value.toFixed(1);
	}

	function effectiveRoundClueRatings() {
		return leaderboardRoundClueRatingOverride ?? $resultsPlayerClueRatings;
	}

	$: shouldShowResultsClueRatings = stage === 'Results' && supportsClueStarsMode;
	$: shouldShowPreviewClueRatings = leaderboardRoundClueRatingOverride !== null;
	$: shouldShowRoundClueRatings = shouldShowResultsClueRatings || shouldShowPreviewClueRatings;

	function displayRoundClueRating(playerName: string, isObserverRow = false) {
		if (
			leaderboardRoundClueRatingOverride === null &&
			(isObserverRow || playerName === activePlayer)
		) {
			return '—';
		}
		const stars = effectiveRoundClueRatings()[playerName];
		return typeof stars === 'number' ? `${stars}★` : '—';
	}

	function hasRoundClueRating(playerName: string, isObserverRow = false) {
		if (
			leaderboardRoundClueRatingOverride === null &&
			(isObserverRow || playerName === activePlayer)
		) {
			return false;
		}
		return typeof effectiveRoundClueRatings()[playerName] === 'number';
	}
</script>

<div class="w-full">
	<div class="card light p-4">
		<h2 class="text-xl">Round {roundNum}</h2>
		{#if supportsBeautyModes || supportsClueStarsMode}
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
		{#if canShowSinceJoined && viewerJoinedRound !== null && activeLeaderboardViewMode !== 'clue_stars'}
			<label class="mt-3 flex items-start gap-3 text-sm">
				<input
					type="checkbox"
					class="mt-0.5 h-4 w-4 cursor-pointer accent-primary-500"
					bind:checked={showSinceJoined}
				/>
				<span>
					Show leaderboard as if the game started from round {viewerJoinedRound}, when you first
					joined.
				</span>
			</label>
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
						<div class="flex min-w-0 flex-auto items-center gap-2">
							<div class="min-w-0 flex-auto">
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
							{#if shouldShowRoundClueRatings}
								<span
									class={`result-clue-rating ${hasRoundClueRating(entry.name) ? 'active' : ''}`}
									title="Round star rating"
								>
									{displayRoundClueRating(entry.name)}
								</span>
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
							<div class="flex min-w-0 flex-auto items-center gap-2">
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
								{#if shouldShowRoundClueRatings}
									<span
										class={`result-clue-rating ${hasRoundClueRating(observerEntry.name, true) ? 'active' : ''}`}
										title="Round star rating"
									>
										{displayRoundClueRating(observerEntry.name, true)}
									</span>
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
								{:else if activeLeaderboardViewMode === 'clue_stars'}
									{formatClueStars(clueStarsForPlayer(observerEntry.name))}
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

	.result-clue-rating {
		display: inline-flex;
		min-width: 2.8rem;
		align-items: center;
		justify-content: center;
		border-radius: 9999px;
		border: 1px solid rgb(255 255 255 / 0.15);
		background: rgb(255 255 255 / 0.06);
		padding: 0.15rem 0.55rem;
		font-size: 0.75rem;
		font-weight: 700;
		color: rgb(226 232 240 / 0.8);
	}

	.result-clue-rating.active {
		border-color: rgb(253 224 71 / 0.45);
		background: linear-gradient(135deg, rgb(250 204 21 / 0.22), rgb(251 191 36 / 0.1));
		color: rgb(254 240 138);
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
