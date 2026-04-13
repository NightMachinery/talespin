<script lang="ts">
	import type GameServer from '$lib/gameServer';
	import { getStellaCardEffectPresentation } from '$lib/stellaCardEffects';
	import type { GameMode, ObserverInfo, PlayerInfo, WinCondition } from '$lib/types';
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
	export let revealedCardChoosers: { [key: string]: string[] } = {};
	export let cardPoints: { [key: string]: number } = {};
	export let darkPlayer = '';
	export let gameMode: GameMode = 'stella';

	$: isObserver = !!observers[name];
	$: isModerator = new Set(moderators).has(name);
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
			<h1 class="text-xl font-semibold">Resonance — Results</h1>
			<p>Clue word: <span class="boujee-text">{clueWord}</span></p>
		</div>
		<div class="card light p-4 space-y-2">
			<button
				class="btn variant-filled w-full"
				disabled={isObserver}
				on:click={() => gameServer.ready()}>Next Round</button
			>
			{#if isModerator}
				<div class="pt-3">
					<button
						class="btn variant-filled w-full"
						on:click={() => gameServer.forceStartNextRound()}>Force next round</button
					>
				</div>
			{/if}
		</div>
	</svelte:fragment>

	<svelte:fragment slot="mobileTop">
		<div class="card light space-y-2 p-4">
			<h1 class="text-xl font-semibold">Resonance — Results</h1>
			<p>{clueWord}</p>
		</div>
	</svelte:fragment>

	<svelte:fragment slot="mobileActions">
		<div class="space-y-2">
			<button
				class="btn variant-filled w-full"
				disabled={isObserver}
				on:click={() => gameServer.ready()}>Next Round</button
			>
			{#if isModerator}
				<div class="pt-3">
					<button
						class="btn variant-filled w-full"
						on:click={() => gameServer.forceStartNextRound()}>Force next round</button
					>
				</div>
			{/if}
		</div>
	</svelte:fragment>

	<div class="flex h-full flex-col">
		<h2 class="mb-2 hidden text-lg font-semibold lg:block">Board</h2>
		<div class="min-h-0 flex-1 overflow-y-auto">
			<Images
				{displayImages}
				selectable={false}
				desktopFitToHeight={true}
				imageAnnotations={revealedCardAnnotations}
				imageChooserOverlays={revealedCardChooserEntries}
				imageHighlightClasses={revealedCardHighlightClasses}
				showIndexOverlay={showVotingCardNumbers}
				indexOverlayPosition="left"
			/>
		</div>
	</div>
</StageShell>
