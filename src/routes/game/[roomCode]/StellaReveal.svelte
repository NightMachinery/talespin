<script lang="ts">
	import type GameServer from '$lib/gameServer';
	import { getStellaCardEffectPresentation } from '$lib/stellaCardEffects';
	import type { GameMode, ObserverInfo, PlayerInfo, WinCondition } from '$lib/types';
	import { onDestroy } from 'svelte';
	import { hideNonSelectedStellaRevealCards } from '$lib/viewOptions';
	import Images from './Images.svelte';
	import StageActionButtons from './StageActionButtons.svelte';
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
	export let beautyEnabled = false;
	export let beautyVotesPerPlayer = 2;
	export let beautyVotesPerPlayerMin = 1;
	export let beautyVotesPerPlayerMax = 1;
	export let beautyAllowDuplicateVotes = false;
	export let beautySplitPointsOnTie = true;
	export let beautyPointsBonus = 2;
	export let beautyPointsBonusMin = 0;
	export let beautyPointsBonusMax = 10;
	export let beautyResultsDisplayMode: import('$lib/types').BeautyResultsDisplayMode = 'combined';
	export let showPreviousResultsDuringStorytellerChoosing = true;
	export let randomizeVotingCardOrderPerViewer = false;
	export let cardsPerHand = 12;
	export let cardsPerHandMin = 1;
	export let cardsPerHandMax = 100;
	export let nominationsPerGuesser = 1;
	export let nominationsPerGuesserMin = 1;
	export let nominationsPerGuesserMax = 1;
	export let bonusCorrectGuessOnThresholdCorrectLoss = true;
	export let bonusDoubleVoteOnThresholdCorrectLoss = true;
	export let bonusThresholdLossTogglesApplyToAllStorytellerLossRounds = true;
	export let showVotingCardNumbers = true;
	export let roundStartDiscardCount = 3;
	export let hintChoosingTimerEnabled = true;
	export let hintChoosingTimerDurationS = 60;
	export let forceHintChoosingTimer = false;
	export let cardChoosingTimerEnabled = true;
	export let cardChoosingTimerDurationS = 30;
	export let votingTimerEnabled = true;
	export let votingTimerDurationS = 180;
	export let beautyTimerEnabled = true;
	export let beautyTimerDurationS = 60;
	export let forceCardChoosingTimer = false;
	export let forceVotingTimer = false;
	export let forceBeautyTimer = false;
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
	export let winCondition: WinCondition = { mode: 'cards_finish' };
	export let clueWord = '';
	export let selectedCards: string[] = [];
	export let selectedCounts: { [key: string]: number } = {};
	export let revealedCards: string[] = [];
	export let revealedCardChoosers: { [key: string]: string[] } = {};
	export let cardPoints: { [key: string]: number } = {};
	export let darkPlayer = '';
	export let gameMode: GameMode = 'stella';

	let latestAnimatedRevealCard = '';
	let latestRevealTimeout: ReturnType<typeof setTimeout> | undefined;
	let previousRevealedCardsKey = '';
	let scoutFlash = false;
	let scoutFlashTimeout: ReturnType<typeof setTimeout> | undefined;
	let previousScoutTurnKey = '';
	let isModerator = false;
	let forceRevealLabel = '';

	$: isObserver = !!observers[name];
	$: isScout = activePlayer === name && !isObserver;
	$: isModerator = new Set(moderators).has(name);
	$: forceRevealLabel = stellaQueueDuringAssociation ? 'Force Next Reveal' : 'Force Random Reveal';
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
			`${presentation.highlightClass} ${card === latestAnimatedRevealCard ? 'stella-card-newly-revealed' : ''}`.trim()
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
	$: revealedCardsKey = revealedCards.join('||');
	$: if (
		revealedCardsKey !== previousRevealedCardsKey &&
		revealedCards.length > 0 &&
		revealedCards.length >= previousRevealedCardsKey.split('||').filter(Boolean).length
	) {
		const previousCount = previousRevealedCardsKey.split('||').filter(Boolean).length;
		if (revealedCards.length > previousCount) {
			latestAnimatedRevealCard = revealedCards[revealedCards.length - 1] ?? '';
			if (latestRevealTimeout) clearTimeout(latestRevealTimeout);
			latestRevealTimeout = setTimeout(() => {
				latestAnimatedRevealCard = '';
			}, 1200);
		}
		previousRevealedCardsKey = revealedCardsKey;
	}
	$: scoutTurnKey = isScout ? `${roundNum}:${activePlayer}:${revealedCards.length}` : '';
	$: if (scoutTurnKey === '') {
		previousScoutTurnKey = '';
	} else if (scoutTurnKey !== previousScoutTurnKey) {
		previousScoutTurnKey = scoutTurnKey;
		scoutFlash = true;
		if (scoutFlashTimeout) clearTimeout(scoutFlashTimeout);
		scoutFlashTimeout = setTimeout(() => {
			scoutFlash = false;
		}, 1400);
	}

	onDestroy(() => {
		if (latestRevealTimeout) clearTimeout(latestRevealTimeout);
		if (scoutFlashTimeout) clearTimeout(scoutFlashTimeout);
	});

	function reveal(card: string) {
		if (!isScout || stellaQueueDuringAssociation) return;
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
	{beautyEnabled}
	{beautyVotesPerPlayer}
	{beautyVotesPerPlayerMin}
	{beautyVotesPerPlayerMax}
	{beautyAllowDuplicateVotes}
	{beautySplitPointsOnTie}
	{beautyPointsBonus}
	{beautyPointsBonusMin}
	{beautyPointsBonusMax}
	{beautyResultsDisplayMode}
	{showPreviousResultsDuringStorytellerChoosing}
	{randomizeVotingCardOrderPerViewer}
	{cardsPerHand}
	{cardsPerHandMin}
	{cardsPerHandMax}
	{nominationsPerGuesser}
	{nominationsPerGuesserMin}
	{nominationsPerGuesserMax}
	{bonusCorrectGuessOnThresholdCorrectLoss}
	{bonusDoubleVoteOnThresholdCorrectLoss}
	{bonusThresholdLossTogglesApplyToAllStorytellerLossRounds}
	{showVotingCardNumbers}
	{roundStartDiscardCount}
	{hintChoosingTimerEnabled}
	{hintChoosingTimerDurationS}
	{forceHintChoosingTimer}
	{cardChoosingTimerEnabled}
	{cardChoosingTimerDurationS}
	{votingTimerEnabled}
	{votingTimerDurationS}
	{beautyTimerEnabled}
	{beautyTimerDurationS}
	{forceCardChoosingTimer}
	{forceVotingTimer}
	{forceBeautyTimer}
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
	{darkPlayer}
	{winCondition}
	{gameMode}
>
	<svelte:fragment slot="leftRail">
		<div class="card light space-y-2 p-4">
			<h1 class="text-xl font-semibold">Resonance — Reveal</h1>
			<p>Clue word: <span class="boujee-text">{clueWord}</span></p>
			<p class:stella-scout-flash={scoutFlash}>
				Scout: <span class="font-semibold">{activePlayer}</span>
			</p>
		</div>
		<div class="card light space-y-2 p-4">
			<h2 class="font-semibold">Selection counts</h2>
			{#each Object.entries(selectedCounts).sort( ([a], [b]) => a.localeCompare(b) ) as [playerName, count]}
				<p>{playerName}: {count}</p>
			{/each}
		</div>
		{#if stellaQueueDuringAssociation}
			<div class="card light space-y-2 p-4">
				<h2 class="font-semibold">Queued reveal</h2>
				<p class="text-sm opacity-80">
					{#if isScout}
						Your queued reveal is playing automatically.
					{:else}
						Waiting for <span class="font-semibold">{activePlayer}</span>'s queued reveal.
					{/if}
				</p>
			</div>
		{:else if isScout}
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
		{#if isModerator}
			<div class="card light p-4">
				<StageActionButtons
					actions={[{ label: forceRevealLabel, onClick: () => gameServer.forceCurrentStage() }]}
				/>
			</div>
		{/if}
	</svelte:fragment>

	<svelte:fragment slot="mobileTop">
		<div class="card light space-y-2 p-4">
			<h1 class="text-xl font-semibold">Resonance — Reveal</h1>
			<p>{clueWord}</p>
			<p class:stella-scout-flash={scoutFlash}>Scout: {activePlayer}</p>
			{#if stellaQueueDuringAssociation}
				<p class="text-sm opacity-80">Queued reveals are playing automatically.</p>
			{:else if isScout}
				<p class="text-sm opacity-80">Tap a highlighted card on the board to reveal it.</p>
			{/if}
		</div>
	</svelte:fragment>

	<svelte:fragment slot="mobileActions">
		{#if isModerator}
			<StageActionButtons
				actions={[{ label: forceRevealLabel, onClick: () => gameServer.forceCurrentStage() }]}
			/>
		{/if}
	</svelte:fragment>

	<div class="flex h-full flex-col">
		<h2 class="mb-2 hidden text-lg font-semibold lg:block">Shared board</h2>
		<div class="min-h-0 flex-1 overflow-y-auto">
			<Images
				displayImages={visibleBoardCards}
				selectedImages={selectedCards}
				selectable={isScout && !stellaQueueDuringAssociation}
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

<style>
	.stella-scout-flash {
		animation: stella-scout-flash 1.4s ease-out;
	}

	@keyframes stella-scout-flash {
		0% {
			transform: scale(1);
			text-shadow: 0 0 0 rgb(96 165 250 / 0);
		}

		25% {
			transform: scale(1.02);
			text-shadow: 0 0 18px rgb(96 165 250 / 0.75);
		}

		100% {
			transform: scale(1);
			text-shadow: 0 0 0 rgb(96 165 250 / 0);
		}
	}
</style>
