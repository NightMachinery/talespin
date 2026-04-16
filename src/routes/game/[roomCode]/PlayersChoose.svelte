<script lang="ts">
	import type GameServer from '$lib/gameServer';
	import type {
		ObserverInfo,
		PlayerInfo,
		PreviousDixitResultsView,
		WinCondition
	} from '$lib/types';
	import {
		previousDixitResultsLeaderboardContext,
		type PreviousDixitResultsLeaderboardContext
	} from '$lib/previousDixitResultsLeaderboard';
	import Images from './Images.svelte';
	import PreviousDixitResultsPreview from './PreviousDixitResultsPreview.svelte';
	import StageActionButtons from './StageActionButtons.svelte';
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
	export let previousDixitResults: PreviousDixitResultsView | null = null;
	export let chosenCard = '';
	export let players: { [key: string]: PlayerInfo } = {};
	export let allowNewPlayersMidgame = true;
	export let storytellerLossComplement = 0;
	export let storytellerLossComplementMin = 0;
	export let storytellerLossComplementMax = 0;
	export let storytellerLossComplementAuto = true;
	export let storytellerPoolEnabled = false;
	export let storytellerPoolActive = false;
	export let storytellerPoolPlayers: string[] = [];
	export let storytellerSuccessPoints = 3;
	export let storytellerSuccessPointsMin = 0;
	export let storytellerSuccessPointsMax = 10;
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
	export let cardsPerHandMax = 18;
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
	export let winCondition: WinCondition = {
		mode: 'points',
		target_points: 10
	};

	type PlayersChooseViewMode = 'results' | 'hand';

	let toastStore = getToastStore();
	let selectedCards: string[] = [];
	let viewMode: PlayersChooseViewMode = 'hand';
	let isObserver = false;
	let isChooser = false;
	let isStoryteller = false;
	let isModerator = false;
	let canAutoObserverify = false;
	let highlightedImages: string[] = [];
	let lastViewResetKey = '';
	let previousResultsLeaderboardContext: PreviousDixitResultsLeaderboardContext | null = null;
	$: isObserver = !!observers[name];
	$: isChooser = activePlayer !== name && !isObserver;
	$: isStoryteller = activePlayer === name && !isObserver;
	$: isModerator = new Set(moderators).has(name);
	$: canAutoObserverify =
		isModerator &&
		Object.entries(players).some(
			([playerName, info]) => playerName !== activePlayer && !info.connected && !info.ready
		);
	$: hasPreviousResultsPreview =
		showPreviousResultsDuringStorytellerChoosing && previousDixitResults !== null;
	$: canToggleResultsView = !isObserver && hasPreviousResultsPreview;
	$: shouldShowPreviousResults =
		hasPreviousResultsPreview && (isObserver || viewMode === 'results');
	$: previousResultsLeaderboardContext = shouldShowPreviousResults
		? previousDixitResultsLeaderboardContext(previousDixitResults)
		: null;
	$: viewResetKey = `${roundNum}:${activePlayer}:${previousDixitResults?.kind ?? 'none'}`;
	$: if (viewResetKey !== lastViewResetKey) {
		lastViewResetKey = viewResetKey;
		viewMode = 'hand';
	}
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
	{storytellerPoolEnabled}
	{storytellerPoolActive}
	{storytellerPoolPlayers}
	{storytellerSuccessPoints}
	{storytellerSuccessPointsMin}
	{storytellerSuccessPointsMax}
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
	{winCondition}
	leaderboardPointChangeStageOverride={previousResultsLeaderboardContext?.stage ?? ''}
	leaderboardPointChangeOverride={previousResultsLeaderboardContext?.pointChange ?? null}
	leaderboardStoryPointChangeOverride={previousResultsLeaderboardContext?.storyPointChange ?? null}
	leaderboardBeautyPointChangeOverride={previousResultsLeaderboardContext?.beautyPointChange ??
		null}
	showMobileActions={isChooser || isModerator || canToggleResultsView}
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
			{#if canToggleResultsView}
				<div class="card light space-y-3 p-4">
					<p class="text-sm font-semibold">Your view</p>
					<div class="grid grid-cols-2 gap-2">
						<button
							class={`btn w-full ${viewMode === 'results' ? 'variant-filled' : 'variant-ghost'}`}
							on:click={() => (viewMode = 'results')}>Previous Results</button
						>
						<button
							class={`btn w-full ${viewMode === 'hand' ? 'variant-filled' : 'variant-ghost'}`}
							on:click={() => (viewMode = 'hand')}>My Cards</button
						>
					</div>
				</div>
			{/if}
		{:else}
			<div class="card light space-y-2 p-4">
				<h1 class="text-2xl">Sit tight!</h1>
				<p>Players are choosing cards that match "{description}"</p>
				{#if isStoryteller && chosenCard}
					<p class="opacity-80">Your chosen card is highlighted below.</p>
				{/if}
				{#if isObserver}
					<p class="opacity-70">
						{#if hasPreviousResultsPreview}
							Review the previous results while you observe.
						{:else}
							You are observing this round.
						{/if}
					</p>
				{:else if canToggleResultsView}
					<p class="opacity-70">
						Review the previous results or switch back to your cards while nominations happen.
					</p>
				{/if}
			</div>
			{#if canToggleResultsView}
				<div class="card light space-y-3 p-4">
					<p class="text-sm font-semibold">Your view</p>
					<div class="grid grid-cols-2 gap-2">
						<button
							class={`btn w-full ${viewMode === 'results' ? 'variant-filled' : 'variant-ghost'}`}
							on:click={() => (viewMode = 'results')}>Previous Results</button
						>
						<button
							class={`btn w-full ${viewMode === 'hand' ? 'variant-filled' : 'variant-ghost'}`}
							on:click={() => (viewMode = 'hand')}>My Cards</button
						>
					</div>
				</div>
			{/if}
		{/if}
		{#if isModerator}
			<div class="card light p-4">
				<StageActionButtons
					actions={[
						{ label: 'Force Random', onClick: () => gameServer.forceCurrentStage() },
						{
							label: 'Auto-observerify',
							disabled: !canAutoObserverify,
							onClick: () => gameServer.autoObserverifyOfflinePendingPlayers()
						}
					]}
				/>
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
		{:else if !isObserver && canToggleResultsView}
			<div class="card light space-y-2 p-4">
				<h1 class="text-2xl">Sit tight!</h1>
				<p>Players are choosing cards that match "{description}"</p>
				{#if isStoryteller && chosenCard}
					<p class="opacity-80">Your chosen card is highlighted below.</p>
				{/if}
				<p class="opacity-70">
					Review the previous results or switch back to your cards while nominations happen.
				</p>
			</div>
		{:else}
			<div class="card light space-y-2 p-4">
				<h1 class="text-2xl">{isObserver ? 'Observing nominations' : 'Sit tight!'}</h1>
				<p>
					{#if isObserver}
						Players are choosing cards that match "{description}".
					{:else}
						Players are choosing cards that match "{description}".
					{/if}
				</p>
				{#if isObserver}
					<p class="opacity-70">
						{#if hasPreviousResultsPreview}
							Review the previous results while you observe.
						{:else}
							You are observing this round.
						{/if}
					</p>
				{/if}
			</div>
		{/if}
	</svelte:fragment>

	<svelte:fragment slot="mobileActions">
		<div class="space-y-4">
			{#if isChooser}
				<button class="btn variant-filled w-full" disabled={!canSubmit} on:click={choose}
					>Choose</button
				>
			{/if}
			{#if canToggleResultsView}
				<div class="grid grid-cols-2 gap-2">
					<button
						class={`btn w-full ${viewMode === 'results' ? 'variant-filled' : 'variant-ghost'}`}
						on:click={() => (viewMode = 'results')}>Previous Results</button
					>
					<button
						class={`btn w-full ${viewMode === 'hand' ? 'variant-filled' : 'variant-ghost'}`}
						on:click={() => (viewMode = 'hand')}>My Cards</button
					>
				</div>
			{/if}
			{#if isModerator}
				<StageActionButtons
					actions={[
						{ label: 'Force Random', onClick: () => gameServer.forceCurrentStage() },
						{
							label: 'Auto-observerify',
							disabled: !canAutoObserverify,
							onClick: () => gameServer.autoObserverifyOfflinePendingPlayers()
						}
					]}
				/>
			{/if}
		</div>
	</svelte:fragment>

	{#if shouldShowPreviousResults && previousDixitResults}
		<PreviousDixitResultsPreview snapshot={previousDixitResults} {showVotingCardNumbers} />
	{:else if !isObserver}
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
	{:else}
		<div class="flex h-full items-center justify-center">
			<div class="card light max-w-md space-y-2 p-4 text-center">
				<h2 class="text-lg font-semibold">Observing card choosing</h2>
				<p class="opacity-80">
					You will see the table once everyone finishes choosing cards for "{description}".
				</p>
			</div>
		</div>
	{/if}
</StageShell>
