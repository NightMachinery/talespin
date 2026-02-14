<script lang="ts">
	import { CARD_IMAGE_ALT_TEXT } from '$lib/cardImageText';
	import { http_host } from '$lib/gameServer';
	import { cardsFitToHeight } from '$lib/viewOptions';
	import type GameServer from '$lib/gameServer';
	import type { ObserverInfo, PlayerInfo, WinCondition } from '$lib/types';
	import StageShell from './StageShell.svelte';

	export let displayImages: string[] = [];
	export let name = '';
	export let creator = '';
	export let moderators: string[] = [];
	export let observers: { [key: string]: ObserverInfo } = {};
	export let activeCard = '';
	export let activePlayer = '';
	export let gameServer: GameServer;
	export let playerToCurrentCards: { [key: string]: string[] } = {};
	export let playerToVotes: { [key: string]: string[] } = {};
	export let players: { [key: string]: PlayerInfo } = {};
	export let allowNewPlayersMidgame = true;
	export let storytellerLossComplement = 0;
	export let storytellerLossComplementMin = 0;
	export let storytellerLossComplementMax = 0;
	export let votesPerGuesser = 1;
	export let votesPerGuesserMin = 1;
	export let votesPerGuesserMax = 1;
	export let cardsPerHand = 6;
	export let cardsPerHandMin = 1;
	export let cardsPerHandMax = 12;
	export let nominationsPerGuesser = 1;
	export let nominationsPerGuesserMin = 1;
	export let nominationsPerGuesserMax = 1;
	export let bonusCorrectGuessOnThresholdCorrectLoss = false;
	export let bonusDoubleVoteOnThresholdCorrectLoss = false;
	export let stage = '';
	export let pointChange: { [key: string]: number } = {};
	export let roundNum = 0;
	export let cardsRemaining = 0;
	export let deckRefillFlashToken = 0;
	export let winCondition: WinCondition = {
		mode: 'points',
		target_points: 10
	};

	let cardToPlayer: { [key: string]: string } = {};
	let cardToVoterCounts: { [key: string]: { [key: string]: number } } = {};
	let isObserver = false;
	$: isObserver = !!observers[name];
	$: resultsDesktopFitClass = $cardsFitToHeight ? 'lg:h-full' : '';
	$: resultsSectionClass = $cardsFitToHeight
		? 'grid w-full grid-cols-2 gap-3 overflow-y-auto lg:h-full lg:grid-cols-3 lg:grid-rows-2 lg:content-stretch'
		: 'grid w-full grid-cols-2 gap-3 overflow-y-auto lg:grid-cols-3 lg:auto-rows-max lg:content-start';
	$: resultsCardClass = (isActiveCard: boolean) =>
		`${isActiveCard ? 'boujee-border' : ''} relative overflow-hidden rounded-lg ${resultsDesktopFitClass}`;
	$: resultsImageClass = `relative w-full object-cover object-center aspect-[2/3] ${resultsDesktopFitClass}`;

	$: {
		cardToPlayer = {};
		cardToVoterCounts = {};
		Object.entries(playerToCurrentCards).forEach(([key, values]) => {
			for (const value of values || []) {
				cardToPlayer[value] = key;
			}
		});

		Object.entries(playerToVotes).forEach(([voter, votes]) => {
			for (const votedCard of votes || []) {
				if (!cardToVoterCounts[votedCard]) {
					cardToVoterCounts[votedCard] = {};
				}
				if (!cardToVoterCounts[votedCard][voter]) {
					cardToVoterCounts[votedCard][voter] = 0;
				}
				cardToVoterCounts[votedCard][voter] += 1;
			}
		});
	}
</script>

<StageShell
	{players}
	{name}
	{creator}
	{moderators}
	{observers}
	{gameServer}
	{stage}
	{allowNewPlayersMidgame}
	{storytellerLossComplement}
	{storytellerLossComplementMin}
	{storytellerLossComplementMax}
	{votesPerGuesser}
	{votesPerGuesserMin}
	{votesPerGuesserMax}
	{cardsPerHand}
	{cardsPerHandMin}
	{cardsPerHandMax}
	{nominationsPerGuesser}
	{nominationsPerGuesserMin}
	{nominationsPerGuesserMax}
	{bonusCorrectGuessOnThresholdCorrectLoss}
	{bonusDoubleVoteOnThresholdCorrectLoss}
	{pointChange}
	{activePlayer}
	{roundNum}
	{cardsRemaining}
	{deckRefillFlashToken}
	{winCondition}
>
	<svelte:fragment slot="leftRail">
		<div class="card light space-y-2 p-4">
			<h1 class="text-xl font-semibold">Round complete</h1>
			<p>Review votes and scores, then continue to the next round.</p>
		</div>
		<div class="card light p-4">
			<button
				class="btn variant-filled w-full"
				disabled={isObserver}
				on:click={() => gameServer.ready()}>Next Round</button
			>
			{#if isObserver}
				<p class="mt-2 text-xs opacity-70">Observers cannot ready up.</p>
			{/if}
		</div>
	</svelte:fragment>

	<svelte:fragment slot="mobileTop">
		<div class="card light space-y-2 p-4">
			<h1 class="text-xl font-semibold">Round complete</h1>
			<p>Review votes and scores, then continue to the next round.</p>
		</div>
	</svelte:fragment>

	<svelte:fragment slot="mobileActions">
		<button
			class="btn variant-filled w-full"
			disabled={isObserver}
			on:click={() => gameServer.ready()}>Next Round</button
		>
	</svelte:fragment>

	<div class="flex h-full flex-col">
		<h2 class="mb-2 hidden text-lg font-semibold lg:block">Round cards</h2>
		<section class={resultsSectionClass}>
			{#each displayImages as image}
				<div class={resultsCardClass(activeCard == image)}>
					<img
						src={`${http_host}/cards/${image}`}
						alt={CARD_IMAGE_ALT_TEXT}
						class={resultsImageClass}
					/>
					{#if cardToVoterCounts[image]}
						<div class="absolute" style="top: 20px; right: 12px;">
							<div class="flex flex-col gap-2">
								{#each Object.entries(cardToVoterCounts[image]) as [voter, count]}
									<div class="bg-success-500 px-1.5 rounded text-black font-bold">
										🔘 {voter}{count > 1 ? ` ×${count}` : ''}
									</div>
								{/each}
							</div>
						</div>
					{/if}
					<div
						style="bottom: 0;"
						class="rounded-tr w-full absolute bg-primary-200 p-0.5 px-2 text-black font-bold"
					>
						{cardToPlayer[image]}'s card
					</div>
				</div>
			{/each}
		</section>
	</div>
</StageShell>

<style>
	@property --bg-angle {
		inherits: false;
		initial-value: 0deg;
		syntax: '<angle>';
	}
	.boujee-border {
		animation: spin 2.5s infinite linear;
		background:
			linear-gradient(to bottom, rgb(var(--color-primary-500)), rgb(var(--color-primary-500)))
				padding-box,
			conic-gradient(from var(--bg-angle) in oklch longer hue, rgb(var(--color-success-500)) 0 0)
				border-box;
		border: 5px solid transparent;
		box-shadow: 0.125rem 0.25rem 0.25rem 0.5rem oklch(0.1 0.37 315 / 0.25);
	}

	@keyframes spin {
		to {
			--bg-angle: 360deg;
		}
	}
</style>
