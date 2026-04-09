<script lang="ts">
	import type GameServer from '$lib/gameServer';
	import type { ObserverInfo, PlayerInfo, WinCondition } from '$lib/types';
	import Images from './Images.svelte';
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
	export let chosenCard = '';
	export let players: { [key: string]: PlayerInfo } = {};
	export let allowNewPlayersMidgame = true;
	export let storytellerLossComplement = 0;
	export let storytellerLossComplementMin = 0;
	export let storytellerLossComplementMax = 0;
	export let storytellerLossComplementAuto = true;
	export let votesPerGuesser = 1;
	export let votesPerGuesserMin = 1;
	export let votesPerGuesserMax = 1;
	export let cardsPerHand = 12;
	export let cardsPerHandMin = 1;
	export let cardsPerHandMax = 18;
	export let nominationsPerGuesser = 1;
	export let nominationsPerGuesserMin = 1;
	export let nominationsPerGuesserMax = 1;
	export let bonusCorrectGuessOnThresholdCorrectLoss = true;
	export let bonusDoubleVoteOnThresholdCorrectLoss = true;
	export let showVotingCardNumbers = true;
	export let roundStartDiscardCount = 3;
	export let hintChoosingTimerEnabled = true;
	export let hintChoosingTimerDurationS = 60;
	export let cardChoosingTimerEnabled = true;
	export let cardChoosingTimerDurationS = 30;
	export let votingTimerEnabled = true;
	export let votingTimerDurationS = 180;
	export let forceCardChoosingTimer = false;
	export let forceVotingTimer = false;
	export let stellaBoardSize = 15;
	export let stellaBoardSizeMin = 1;
	export let stellaBoardSizeMax = 100;
	export let stellaSelectionMin = 1;
	export let stellaSelectionMax = 10;
	export let stellaSelectionCountMin = 1;
	export let stellaSelectionCountMax = 15;
	export let stellaWordPackPresetNames: string[] = [];
	export let stellaSelectedWordPackName = '';
	export let stellaWordPackIsUnsaved = false;
	export let stellaQueueDuringAssociation = true;
	export let stellaQueuedRevealMode: 'animated' | 'fast' = 'animated';
	export let stellaScoutTimerEnabled = true;
	export let stellaScoutTimerDurationS = 10;
	export let forceStellaScoutTimer = false;
	export let serverTimeMs: number | null = null;
	export let currentStageDeadlineS: number | null = null;
	export let votingWrongCardDisableDistribution: number[] = [1];
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
	let isStoryteller = false;
	let highlightedImages: string[] = [];
	$: isObserver = !!observers[name];
	$: isChooser = activePlayer !== name && !isObserver;
	$: isStoryteller = activePlayer === name && !isObserver;
	$: effectiveNominationsPerGuesser = Math.max(
		1,
		Math.min(nominationsPerGuesser, Math.max(nominationsPerGuesserMax, 1))
	);
	$: canSubmit = isChooser && selectedCards.length === effectiveNominationsPerGuesser;
	$: highlightedImages = isChooser
		? selectedCards
		: isStoryteller && chosenCard
			? [chosenCard]
			: [];
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

	function handleCardSelect(event: CustomEvent<string>) {
		toggleCard(event.detail);
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
	{storytellerLossComplementAuto}
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
	{showVotingCardNumbers}
	{roundStartDiscardCount}
	{hintChoosingTimerEnabled}
	{hintChoosingTimerDurationS}
	{cardChoosingTimerEnabled}
	{cardChoosingTimerDurationS}
	{votingTimerEnabled}
	{votingTimerDurationS}
	{forceCardChoosingTimer}
	{forceVotingTimer}
	{stellaBoardSize}
	{stellaBoardSizeMin}
	{stellaBoardSizeMax}
	{stellaSelectionMin}
	{stellaSelectionMax}
	{stellaSelectionCountMin}
	{stellaSelectionCountMax}
	{stellaWordPackPresetNames}
	{stellaSelectedWordPackName}
	{stellaWordPackIsUnsaved}
	{stellaQueueDuringAssociation}
	{stellaQueuedRevealMode}
	{stellaScoutTimerEnabled}
	{stellaScoutTimerDurationS}
	{forceStellaScoutTimer}
	{serverTimeMs}
	{currentStageDeadlineS}
	{votingWrongCardDisableDistribution}
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
				{#if isStoryteller && chosenCard}
					<p class="opacity-80">Your chosen card is highlighted below.</p>
				{/if}
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
				{#if isStoryteller && chosenCard}
					<p class="opacity-80">Your chosen card is highlighted below.</p>
				{/if}
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
		<div class="min-h-0 flex-1 overflow-y-auto">
			<Images
				{displayImages}
				selectedImages={highlightedImages}
				selectable={isChooser}
				mode="hand"
				on:select={handleCardSelect}
			/>
		</div>
	</div>
</StageShell>
