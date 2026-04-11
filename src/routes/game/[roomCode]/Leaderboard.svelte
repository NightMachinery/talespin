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

	export let players: { [key: string]: PlayerInfo } = {};
	export let observers: { [key: string]: ObserverInfo } = {};
	export let name = '';
	export let moderators: string[] = [];
	export let gameServer: GameServer;
	export let stage = '';
	export let activePlayer = '';
	export let pointChange: { [key: string]: number } = {};
	export let roundNum: number;
	export let cardsRemaining = 0;
	export let deckRefillFlashToken = 0;
	export let darkPlayer = '';
	export let gameMode: GameMode = 'dixit_plus';
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

	$: isModerator = new Set(moderators).has(name);
	$: supportsBeautyModes = gameMode === 'dixit_plus';
	$: activeLeaderboardViewMode = supportsBeautyModes ? $leaderboardViewMode : 'total';
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
			stage === 'Results' ||
			stage === 'BeautyResults' ||
			stage === 'StellaReveal' ||
			stage === 'StellaResults'
		);
	}

	function beautyPointsForPlayer(playerName: string) {
		return $memberBeautyPoints[playerName] ?? 0;
	}

	function displayedPointChangeForPlayer(playerName: string) {
		if (!supportsBeautyModes) {
			return pointChange[playerName] ?? 0;
		}
		switch (activeLeaderboardViewMode) {
			case 'story_only':
				if (stage === 'BeautyResults') {
					return 0;
				}
				return $storytellerLeaderboardPointChange[playerName] ?? 0;
			case 'beauty_only':
				return $beautyLeaderboardPointChange[playerName] ?? 0;
			case 'combined':
				return null;
			case 'total':
			default:
				return pointChange[playerName] ?? 0;
		}
	}

	function handleViewModeChange(event: Event) {
		const select = event.currentTarget as HTMLSelectElement;
		setLeaderboardViewModePreference(select.value as LeaderboardViewMode);
	}

	function pushCurrentViewToRoom() {
		gameServer.setLeaderboardViewModeDefault($leaderboardViewMode);
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
					class="mb-2 flex items-center justify-end gap-2 pr-1 text-[11px] font-semibold uppercase tracking-wide opacity-60"
				>
					<span class="score-column" style={widthStyle(digitWidths.total)}>T</span>
					<span class="score-column" style={widthStyle(digitWidths.story)}>S</span>
					<span class="score-column" style={widthStyle(digitWidths.beauty)}>B</span>
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
								<span class="mr-2 opacity-50">(+{displayedPointChangeForPlayer(entry.name)})</span>
							{/if}
							{#if activeLeaderboardViewMode === 'combined'}
								<div class="flex items-center justify-end gap-2">
									<span class="score-column" style={widthStyle(digitWidths.total)}
										>{entry.breakdown.total}</span
									>
									<span class="score-column opacity-85" style={widthStyle(digitWidths.story)}
										>{entry.breakdown.story}</span
									>
									<span class="score-column opacity-85" style={widthStyle(digitWidths.beauty)}
										>{entry.breakdown.beauty}</span
									>
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
						class="mb-2 mt-1 flex items-center justify-end gap-2 pr-1 text-[11px] font-semibold uppercase tracking-wide opacity-60"
					>
						<span class="score-column" style={widthStyle(digitWidths.total)}>T</span>
						<span class="score-column" style={widthStyle(digitWidths.story)}>S</span>
						<span class="score-column" style={widthStyle(digitWidths.beauty)}>B</span>
					</div>
				{/if}
				<div class="mt-1 space-y-1">
					{#each adjustedObserverEntries as observerEntry}
						<div
							class={`flex items-center justify-between gap-2 ${!observerEntry.info.connected ? 'opacity-50' : ''}`}
						>
							<div class="min-w-0 flex-auto break-words">
								{observerEntry.name}
								{#if observerEntry.info.join_requested || observerEntry.info.auto_join_on_next_round}
									<span class="opacity-70"> (joining next round)</span>
								{/if}
								{#if !observerEntry.info.connected}
									<span class="opacity-70"> ({OFFLINE_STATUS_LABEL})</span>
								{/if}
							</div>
							<div class="shrink-0 text-right font-mono tabular-nums">
								{#if activeLeaderboardViewMode !== 'combined' && shouldShowPointChange() && observerEntry.displayPointChange !== 0}
									<span class="mr-2 opacity-50">(+{observerEntry.displayPointChange})</span>
								{/if}
								{#if observerEntry.breakdown === null}
									NA
								{:else if activeLeaderboardViewMode === 'combined'}
									<div class="flex items-center justify-end gap-2">
										<span class="score-column" style={widthStyle(digitWidths.total)}
											>{observerEntry.breakdown.total}</span
										>
										<span class="score-column opacity-85" style={widthStyle(digitWidths.story)}
											>{observerEntry.breakdown.story}</span
										>
										<span class="score-column opacity-85" style={widthStyle(digitWidths.beauty)}
											>{observerEntry.breakdown.beauty}</span
										>
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
