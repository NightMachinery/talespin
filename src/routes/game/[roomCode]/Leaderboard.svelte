<script lang="ts">
	import { onDestroy } from 'svelte';
	import type GameServer from '$lib/gameServer';
	import {
		beautyLeaderboardPointChange,
		leaderboardExcludeBeauty,
		memberBeautyPoints,
		roomLeaderboardExcludeBeautyDefault,
		setLeaderboardExcludeBeautyPreference,
		storytellerLeaderboardPointChange
	} from '$lib/mostBeautiful';
	import type { ObserverInfo, PlayerInfo, WinCondition } from '$lib/types';
	import { OFFLINE_STATUS_LABEL } from '$lib/presence';
	import { rankEntriesByPoints, type RankedPlayerEntry } from '$lib/ranking';
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
	export let winCondition: WinCondition = {
		mode: 'points',
		target_points: 10
	};

	let rankedPlayers: RankedPlayerEntry[] = [];
	let winConditionLabel = '';
	let previousFlashToken = 0;
	let cardsRemainingFlashing = false;
	let cardsRemainingFlashTimeout: ReturnType<typeof setTimeout> | undefined = undefined;
	let sortedObserverEntries: [string, ObserverInfo][] = [];
	let adjustedObserverEntries: Array<{
		name: string;
		info: ObserverInfo;
		displayPoints: number | null;
		displayPointChange: number;
	}> = [];

	$: {
		rankedPlayers = rankEntriesByPoints(
			Object.entries(players).map(([entryName, info]) => ({
				name: entryName,
				points: displayedPointsForPlayer(entryName, info.points)
			}))
		);
	}
	$: sortedObserverEntries = Object.entries(observers).sort(([a], [b]) => a.localeCompare(b));
	$: adjustedObserverEntries = sortedObserverEntries.map(([observerName, info]) => ({
		name: observerName,
		info,
		displayPoints:
			info.points === null ? null : displayedPointsForPlayer(observerName, info.points ?? 0),
		displayPointChange: displayedPointChangeForPlayer(observerName)
	}));

	$: winConditionLabel = formatWinCondition(winCondition);
	$: isModerator = new Set(moderators).has(name);

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

	function formatObserverPoints(points: number | null) {
		return points === null ? 'NA' : `${points}`;
	}

	function beautyPointsForPlayer(playerName: string) {
		return $memberBeautyPoints[playerName] ?? 0;
	}

	function displayedPointsForPlayer(playerName: string, points: number) {
		return $leaderboardExcludeBeauty ? points - beautyPointsForPlayer(playerName) : points;
	}

	function displayedPointChangeForPlayer(playerName: string) {
		if (!$leaderboardExcludeBeauty) {
			return pointChange[playerName] ?? 0;
		}
		if (stage === 'Results') {
			return $storytellerLeaderboardPointChange[playerName] ?? 0;
		}
		if (stage === 'BeautyResults') {
			return 0;
		}
		return pointChange[playerName] ?? 0;
	}

	function handleExcludeBeautyToggle(event: Event) {
		const input = event.currentTarget as HTMLInputElement;
		setLeaderboardExcludeBeautyPreference(input.checked);
	}

	function handleRoomDefaultToggle(event: Event) {
		const input = event.currentTarget as HTMLInputElement;
		gameServer.setLeaderboardExcludeBeautyDefault(input.checked);
	}
</script>

<div class="w-full">
	<div class="card light p-4">
		<h2 class="text-xl">Round {roundNum}</h2>
		<div class="mt-3 space-y-2 text-sm">
			<label class="flex items-start gap-3">
				<input
					type="checkbox"
					class="mt-0.5 h-4 w-4 cursor-pointer accent-primary-500"
					checked={$leaderboardExcludeBeauty}
					on:change={handleExcludeBeautyToggle}
				/>
				<span>Exclude beauty votes from leaderboard</span>
			</label>
			{#if isModerator && stage !== 'End'}
				<label class="flex items-start gap-3 text-xs opacity-80">
					<input
						type="checkbox"
						class="mt-0.5 h-4 w-4 cursor-pointer accent-primary-500"
						checked={$roomLeaderboardExcludeBeautyDefault}
						on:change={handleRoomDefaultToggle}
					/>
					<span>Force-push this beauty toggle default to everyone in the room</span>
				</label>
			{/if}
		</div>
		<div class="mt-3">
			<div>
				{#each rankedPlayers as entry}
					<div class="flex items-center justify-between gap-2">
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
						<div class="shrink-0">
							{#if shouldShowPointChange() && displayedPointChangeForPlayer(entry.name) !== 0}
								<span class="opacity-50">(+{displayedPointChangeForPlayer(entry.name)})</span>
							{/if}
							{entry.points}
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
							<div class="shrink-0">
								{#if shouldShowPointChange() && observerEntry.displayPointChange !== 0}
									<span class="opacity-50">(+{observerEntry.displayPointChange})</span>
								{/if}
								{formatObserverPoints(observerEntry.displayPoints)}
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
