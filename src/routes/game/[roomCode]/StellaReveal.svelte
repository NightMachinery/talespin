<script lang="ts">
	import type GameServer from '$lib/gameServer';
	import { getStellaCardEffectPresentation } from '$lib/stellaCardEffects';
	import type { GameMode, ObserverInfo, PlayerInfo, WinCondition } from '$lib/types';
	import { hideNonSelectedStellaRevealCards } from '$lib/viewOptions';
	import Images from './Images.svelte';
	import StageShell from './StageShell.svelte';

	export let displayImages: string[] = [];
	export let name = '';
	export let creator = '';
	export let moderators: string[] = [];
	export let observers: { [key: string]: ObserverInfo } = {};
	export let activePlayer = '';
	export let gameServer: GameServer;
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
	export let cardsPerHandMax = 100;
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
	export let stellaWordPackWords: string[] = [];
	export let serverTimeMs: number | null = null;
	export let currentStageDeadlineS: number | null = null;
	export let votingWrongCardDisableDistribution: number[] = [1];
	export let stage = '';
	export let pointChange: { [key: string]: number } = {};
	export let roundNum = 0;
	export let cardsRemaining = 0;
	export let deckRefillFlashToken = 0;
	export let winCondition: WinCondition = { mode: 'cards_finish' };
	export let clueWord = '';
	export let selectedCards: string[] = [];
	export let selectedCounts: { [key: string]: number } = {};
	export let revealedCards: string[] = [];
	export let revealedCardChoosers: { [key: string]: string[] } = {};
	export let cardPoints: { [key: string]: number } = {};
	export let darkPlayer = '';
	export let gameMode: GameMode = 'stella';

	$: isObserver = !!observers[name];
	$: isScout = activePlayer === name && !isObserver;
	$: revealableCards = selectedCards.filter((card) => !revealedCards.includes(card));
	$: selectedCardSet = new Set(selectedCards);
	$: visibleBoardCards =
		isScout && $hideNonSelectedStellaRevealCards && selectedCards.length > 0
			? displayImages.filter((card) => selectedCardSet.has(card))
			: displayImages;
	$: visibleBoardIndexOverlayLabels =
		isScout && $hideNonSelectedStellaRevealCards && selectedCards.length > 0
			? displayImages.reduce<Array<string | number>>((labels, card, index) => {
					if (selectedCardSet.has(card)) labels.push(index + 1);
					return labels;
				}, [])
			: [];
	$: revealedCardChooserEntries = Object.fromEntries(
		Object.entries(revealedCardChoosers).map(([card, choosers]) => [
			card,
			choosers.map((chooser) => ({ name: chooser }))
		])
	);
	$: revealedCardEffectPresentation = Object.fromEntries(
		Object.entries(cardPoints).map(([card, points]) => [
			card,
			getStellaCardEffectPresentation(points)
		])
	);
	$: revealedCardHighlightClasses = Object.fromEntries(
		Object.entries(revealedCardEffectPresentation).map(([card, presentation]) => [
			card,
			presentation.highlightClass
		])
	);
	$: revealedCardAnnotations = Object.entries(revealedCardEffectPresentation).reduce<
		Record<string, NonNullable<(typeof revealedCardEffectPresentation)[string]['annotation']>>
	>((annotations, [card, presentation]) => {
		if (presentation.annotation) {
			annotations[card] = presentation.annotation;
		}
		return annotations;
	}, {});

	function reveal(card: string) {
		if (!isScout) return;
		gameServer.revealStellaCard(card);
	}

	function handleRevealSelect(event: CustomEvent<string>) {
		reveal(event.detail);
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
	{stellaWordPackWords}
	{serverTimeMs}
	{currentStageDeadlineS}
	{votingWrongCardDisableDistribution}
	{pointChange}
	{activePlayer}
	{roundNum}
	{cardsRemaining}
	{deckRefillFlashToken}
	{darkPlayer}
	{winCondition}
	{gameMode}
>
	<svelte:fragment slot="leftRail">
		<div class="card light space-y-2 p-4">
			<h1 class="text-xl font-semibold">Resonance — Reveal</h1>
			<p>Clue word: <span class="boujee-text">{clueWord}</span></p>
			<p>Scout: <span class="font-semibold">{activePlayer}</span></p>
		</div>
		<div class="card light space-y-2 p-4">
			<h2 class="font-semibold">Selection counts</h2>
			{#each Object.entries(selectedCounts).sort( ([a], [b]) => a.localeCompare(b) ) as [playerName, count]}
				<p>{playerName}: {count}</p>
			{/each}
		</div>
		{#if isScout}
			<div class="card light space-y-2 p-4">
				<h2 class="font-semibold">Reveal from the board</h2>
				<p class="text-sm opacity-80">Tap one of your highlighted unrevealed cards on the board.</p>
			</div>
		{:else}
			<div class="card light space-y-2 p-4">
				<h2 class="font-semibold">Scout action</h2>
				<p class="text-sm opacity-80">
					Waiting for <span class="font-semibold">{activePlayer}</span> to reveal one highlighted card.
				</p>
			</div>
		{/if}
	</svelte:fragment>

	<svelte:fragment slot="mobileTop">
		<div class="card light space-y-2 p-4">
			<h1 class="text-xl font-semibold">Resonance — Reveal</h1>
			<p>{clueWord}</p>
			<p>Scout: {activePlayer}</p>
			{#if isScout}
				<p class="text-sm opacity-80">Tap a highlighted card on the board to reveal it.</p>
			{/if}
		</div>
	</svelte:fragment>

	<div class="flex h-full flex-col">
		<h2 class="mb-2 hidden text-lg font-semibold lg:block">Shared board</h2>
		<div class="min-h-0 flex-1 overflow-y-auto">
			<Images
				displayImages={visibleBoardCards}
				selectedImages={selectedCards}
				selectable={isScout}
				selectableImages={revealableCards}
				desktopFitToHeight={true}
				imageAnnotations={revealedCardAnnotations}
				imageChooserOverlays={revealedCardChooserEntries}
				imageHighlightClasses={revealedCardHighlightClasses}
				showIndexOverlay={showVotingCardNumbers}
				indexOverlayPosition="left"
				indexOverlayLabels={visibleBoardIndexOverlayLabels}
				on:select={handleRevealSelect}
			/>
		</div>
	</div>
</StageShell>
