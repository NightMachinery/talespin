<script lang="ts">
	import { onDestroy } from 'svelte';
	import type { ObserverInfo, PlayerInfo, WinCondition } from '$lib/types';
	import { OFFLINE_STATUS_LABEL } from '$lib/presence';

	export let players: { [key: string]: PlayerInfo } = {};
	export let observers: { [key: string]: ObserverInfo } = {};
	// export let name = '';
	export let stage = '';
	export let activePlayer = '';
	export let pointChange: { [key: string]: number } = {};
	export let roundNum: number;
	export let cardsRemaining = 0;
	export let deckRefillFlashToken = 0;
	export let winCondition: WinCondition = {
		mode: 'points',
		target_points: 10
	};

	let sortedPlayersList: string[] = [];
	let winConditionLabel = '';
	let previousFlashToken = 0;
	let cardsRemainingFlashing = false;
	let cardsRemainingFlashTimeout: ReturnType<typeof setTimeout> | undefined = undefined;

	$: {
		sortedPlayersList = Object.keys(players).sort((a, b) => {
			return players[b].points - players[a].points;
		});
	}

	$: {
		if (winCondition.mode === 'points') {
			winConditionLabel = `First to ${winCondition.target_points} points!`;
		} else if (winCondition.mode === 'cycles') {
			winConditionLabel = `${winCondition.target_cycles} full storyteller cycle${winCondition.target_cycles === 1 ? '' : 's'}`;
		} else {
			winConditionLabel = 'Play until cards finish';
		}
	}

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
</script>

<div class="w-full">
	<div class="card light p-4">
		<h2 class="text-xl">Round {roundNum}</h2>
		<div>
			{#each sortedPlayersList as player, i}
				<div class="flex items-center justify-between gap-2">
					<div class="flex-auto">
						{i + 1}.
						<span
							class={`${player === activePlayer ? 'boujee-text' : ''} ${!players[player].connected ? 'opacity-50 grayscale' : ''}`}
						>
							{player}
						</span>
						{#if !players[player].connected}
							<span class="text-error-500">({OFFLINE_STATUS_LABEL})</span>
						{/if}

						{#if stage === 'Joining' || ((stage === 'PlayersChoose' || stage === 'Voting') && player !== activePlayer) || stage === 'Results'}
							{#if players[player].ready}
								<span>🟢</span>
							{:else}
								<span>🔴</span>
							{/if}
						{/if}
					</div>
					<div class="shrink-0">
						{#if stage === 'Results' && typeof pointChange[player] === 'number' && pointChange[player] !== 0}
							<span class="opacity-50">(+{pointChange[player]})</span>
						{/if}
						{players[player].points}
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
				<ul class="ml-4 list-disc">
					{#each Object.entries(observers).sort( ([a], [b]) => a.localeCompare(b) ) as [observerName, info]}
						<li class={!info.connected ? 'opacity-50' : ''}>
							{observerName}
							{#if info.join_requested || info.auto_join_on_next_round}
								<span class="opacity-70"> (joining next round)</span>
							{/if}
							{#if !info.connected}
								<span class="opacity-70"> ({OFFLINE_STATUS_LABEL})</span>
							{/if}
						</li>
					{/each}
				</ul>
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
