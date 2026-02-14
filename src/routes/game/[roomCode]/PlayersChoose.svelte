<script lang="ts">
	import type GameServer from '$lib/gameServer';
	import { CARD_IMAGE_ALT_TEXT } from '$lib/cardImageText';
	import { http_host } from '$lib/gameServer';
	import type { ObserverInfo, PlayerInfo, WinCondition } from '$lib/types';
	import { cardsFitToHeight } from '$lib/viewOptions';
	import StageShell from './StageShell.svelte';
	import { getToastStore } from '@skeletonlabs/skeleton';

	export let displayImages: string[] = [];
	export let name = '';
	export let creator = '';
	export let moderators: string[] = [];
	export let observers: { [key: string]: ObserverInfo } = {};
	export let activePlayer = '';
	export let gameServer: GameServer;
	export let description = '';
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

	let toastStore = getToastStore();
	let selectedCards: string[] = [];
	let isObserver = false;
	let isChooser = false;
	$: isObserver = !!observers[name];
	$: isChooser = activePlayer !== name && !isObserver;
	$: effectiveNominationsPerGuesser = Math.max(
		1,
		Math.min(nominationsPerGuesser, Math.max(nominationsPerGuesserMax, 1))
	);
	$: canSubmit = isChooser && selectedCards.length === effectiveNominationsPerGuesser;
	$: handDesktopFitEnabled = $cardsFitToHeight;
	$: handDesktopRowCount = Math.max(2, Math.ceil(Math.max(displayImages?.length ?? 0, 1) / 3));
	$: handSectionClass = handDesktopFitEnabled
		? 'hand-fit-grid grid w-full grid-cols-2 gap-3 overflow-visible p-1 lg:h-full lg:grid-cols-3 lg:content-stretch'
		: 'grid w-full grid-cols-2 gap-3 overflow-visible p-1 lg:grid-cols-3 lg:auto-rows-max lg:content-start';
	$: handImageClass = handDesktopFitEnabled
		? 'pointer-events-none w-full rounded-lg object-cover object-center aspect-[2/3] lg:h-full transition-all duration-150 ease-out'
		: 'pointer-events-none w-full rounded-lg object-cover object-center aspect-[2/3] transition-all duration-150 ease-out';
	$: handButtonClass = handDesktopFitEnabled
		? 'card-hover-source group relative block w-full overflow-visible rounded-lg focus-visible:outline-none lg:h-full'
		: 'card-hover-source group relative block w-full overflow-visible rounded-lg focus-visible:outline-none';
	$: handDesktopFitStyle = handDesktopFitEnabled
		? `--hand-desktop-rows: ${handDesktopRowCount};`
		: '';
	$: {
		const allowed = new Set(displayImages);
		const filtered = selectedCards.filter((card) => allowed.has(card));
		if (filtered.length !== selectedCards.length) {
			selectedCards = filtered;
		}
	}
	$: if (selectedCards.length > effectiveNominationsPerGuesser) {
		selectedCards = selectedCards.slice(selectedCards.length - effectiveNominationsPerGuesser);
	}

	if (name !== activePlayer && !isObserver) {
		toastStore.trigger({
			message: '👉 Your turn!',
			autohide: true,
			timeout: 5000
		});
	}

	function choose() {
		if (!canSubmit) return;
		gameServer.playersChoose(selectedCards);
		toastStore.trigger({
			message: '👌 Locked in!',
			autohide: true,
			timeout: 2500
		});
	}

	function toggleCard(card: string) {
		if (!isChooser) return;
		if (selectedCards.includes(card)) {
			selectedCards = selectedCards.filter((value) => value !== card);
			return;
		}
		let next = [...selectedCards, card];
		while (next.length > effectiveNominationsPerGuesser) {
			next.shift();
		}
		selectedCards = next;
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
	showMobileActions={isChooser}
>
	<svelte:fragment slot="leftRail">
		{#if isChooser}
			<div class="card light space-y-2 p-4">
				<h1 class="text-xl font-semibold">Your turn!</h1>
				<p>
					Choose {effectiveNominationsPerGuesser} card{effectiveNominationsPerGuesser === 1
						? ''
						: 's'} that <span class="boujee-text">{activePlayer}</span> would put for "{description}"
				</p>
				<p class="text-xs opacity-80">
					Selected: {selectedCards.length}/{effectiveNominationsPerGuesser}
				</p>
			</div>
			<div class="card light p-4">
				<button class="btn variant-filled w-full" disabled={!canSubmit} on:click={choose}
					>Choose</button
				>
			</div>
		{:else}
			<div class="card light space-y-2 p-4">
				<h1 class="text-2xl">Sit tight!</h1>
				<p>Players are choosing cards that match "{description}"</p>
				{#if isObserver}
					<p class="opacity-70">You are observing this round.</p>
				{/if}
			</div>
		{/if}
	</svelte:fragment>

	<svelte:fragment slot="mobileTop">
		{#if isChooser}
			<div class="card light space-y-2 p-4">
				<h1 class="text-xl font-semibold">Your turn!</h1>
				<p>
					Choose {effectiveNominationsPerGuesser} card{effectiveNominationsPerGuesser === 1
						? ''
						: 's'} that <span class="boujee-text">{activePlayer}</span> would put for "{description}"
				</p>
				<p class="text-xs opacity-80">
					Selected: {selectedCards.length}/{effectiveNominationsPerGuesser}
				</p>
			</div>
		{:else}
			<div class="card light space-y-2 p-4">
				<h1 class="text-2xl">Sit tight!</h1>
				<p>Players are choosing cards that match "{description}"</p>
				{#if isObserver}
					<p class="opacity-70">You are observing this round.</p>
				{/if}
			</div>
		{/if}
	</svelte:fragment>

	<svelte:fragment slot="mobileActions">
		{#if isChooser}
			<button class="btn variant-filled w-full" disabled={!canSubmit} on:click={choose}
				>Choose</button
			>
		{/if}
	</svelte:fragment>

	<div class="flex h-full flex-col">
		<h2 class="mb-2 hidden text-lg font-semibold lg:block">Your hand</h2>
		<section class={handSectionClass} style={handDesktopFitStyle}>
			{#each displayImages as image}
				<button
					type="button"
					class={`${handButtonClass} ${!isChooser ? 'cursor-default' : ''}`}
					disabled={!isChooser}
					on:click={() => toggleCard(image)}
				>
					<img
						class={`${handImageClass} ${
							selectedCards.includes(image)
								? 'brightness-105 ring-4 ring-white shadow-xlg'
								: 'card-hover-target cursor-pointer group-focus-visible:ring-2 group-focus-visible:ring-white/85 group-focus-visible:shadow-[0_0_0_2px_rgba(255,255,255,0.22),0_16px_30px_rgba(0,0,0,0.38)]'
						}`}
						src={`${http_host}/cards/${image}`}
						alt={CARD_IMAGE_ALT_TEXT}
					/>
				</button>
			{/each}
		</section>
	</div>
</StageShell>

<style>
	@media (min-width: 1024px) {
		.hand-fit-grid {
			grid-template-rows: repeat(var(--hand-desktop-rows, 2), minmax(0, 1fr));
		}
	}
</style>
