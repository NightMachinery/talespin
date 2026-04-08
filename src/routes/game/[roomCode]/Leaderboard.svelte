<script lang="ts">
	import { onDestroy } from 'svelte';
	import type { ObserverInfo, PlayerInfo, WinCondition } from '$lib/types';
	import { OFFLINE_STATUS_LABEL } from '$lib/presence';
	import { rankPlayersByPoints, type RankedPlayerEntry } from '$lib/ranking';
	import { formatWinCondition } from '$lib/winCondition';

	export let players: { [key: string]: PlayerInfo } = {};
	export let observers: { [key: string]: ObserverInfo } = {};
	// export let name = '';
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

	$: {
		rankedPlayers = rankPlayersByPoints(players);
	}
	$: sortedObserverEntries = Object.entries(observers).sort(([a], [b]) => a.localeCompare(b));

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
			stage === 'Results' ||
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
		return stage === 'Results' || stage === 'StellaReveal' || stage === 'StellaResults';
	}

	function formatObserverPoints(points: number | null) {
		return points === null ? 'NA' : `${points}`;
	}
</script>

<div class="w-full">
	<div class="card light p-4">
		<h2 class="text-xl">Round {roundNum}</h2>
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
						{#if shouldShowPointChange() && typeof pointChange[entry.name] === 'number' && pointChange[entry.name] !== 0}
							<span class="opacity-50">(+{pointChange[entry.name]})</span>
						{/if}
						{entry.points}
					</div>
				</div>
			{/each}
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
					{#each sortedObserverEntries as [observerName, info]}
						<div
							class={`flex items-center justify-between gap-2 ${!info.connected ? 'opacity-50' : ''}`}
						>
							<div class="min-w-0 flex-auto break-words">
								{observerName}
								{#if info.join_requested || info.auto_join_on_next_round}
									<span class="opacity-70"> (joining next round)</span>
								{/if}
								{#if !info.connected}
									<span class="opacity-70"> ({OFFLINE_STATUS_LABEL})</span>
								{/if}
							</div>
							<div class="shrink-0">
								{#if shouldShowPointChange() && typeof pointChange[observerName] === 'number' && pointChange[observerName] !== 0}
									<span class="opacity-50">(+{pointChange[observerName]})</span>
								{/if}
								{formatObserverPoints(info.points)}
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
