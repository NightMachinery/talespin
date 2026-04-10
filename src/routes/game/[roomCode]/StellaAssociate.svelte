<script lang="ts">
	import { getToastStore } from '@skeletonlabs/skeleton';
	import type GameServer from '$lib/gameServer';
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
	export let beautyVotesPerPlayer = 1;
	export let beautyVotesPerPlayerMin = 1;
	export let beautyVotesPerPlayerMax = 1;
	export let beautyAllowDuplicateVotes = false;
	export let beautyPointsBonus = 2;
	export let beautyPointsBonusMin = 0;
	export let beautyPointsBonusMax = 10;
	export let beautyResultsDisplayMode: import('$lib/types').BeautyResultsDisplayMode = 'combined';
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
	export let winCondition: WinCondition = { mode: 'cards_finish' };
	export let clueWord = '';
	export let selectedCards: string[] = [];
	export let gameMode: GameMode = 'stella';

	let localSelectedCards: string[] = [];
	let syncedRoundKey = '';
	let syncedSelectedKey = '';
	let draggedQueueCard = '';
	const toastStore = getToastStore();
	$: isObserver = !!observers[name];
	$: isModerator = new Set(moderators).has(name);
	$: canSubmit =
		!isObserver &&
		localSelectedCards.length >= stellaSelectionMin &&
		localSelectedCards.length <= stellaSelectionMax;
	$: hasLockedSelection = selectedCards.length > 0;
	$: selectionDirty = !sameSelections(localSelectedCards, selectedCards);
	$: submitLabel = hasLockedSelection
		? stellaQueueDuringAssociation
			? 'Update locked queue'
			: 'Update locked selections'
		: stellaQueueDuringAssociation
			? 'Lock queue'
			: 'Lock selections';
	$: roundKey = `${roundNum}::${clueWord}::${displayImages.join('||')}`;
	$: selectedKey = selectedCards.join('||');
	$: selectedCardQueueBadges = Object.fromEntries(
		localSelectedCards.map((card, index) => [card, index + 1])
	);
	$: boardIndexByCard = Object.fromEntries(displayImages.map((card, index) => [card, index + 1]));
	$: if (roundKey !== syncedRoundKey) {
		syncedRoundKey = roundKey;
		syncedSelectedKey = selectedKey;
		localSelectedCards = [...selectedCards];
	} else if (selectedKey !== syncedSelectedKey) {
		syncedSelectedKey = selectedKey;
		localSelectedCards = [...selectedCards];
	}

	function sameSelections(left: string[], right: string[]) {
		return left.length === right.length && left.every((value, index) => value === right[index]);
	}

	function toggleCard(card: string) {
		if (isObserver) return;
		if (localSelectedCards.includes(card)) {
			localSelectedCards = localSelectedCards.filter((value) => value !== card);
		} else {
			localSelectedCards = [...localSelectedCards, card];
		}
	}

	function handleSelect(event: CustomEvent<string>) {
		toggleCard(event.detail);
	}

	function moveQueueCard(card: string, direction: -1 | 1) {
		const currentIndex = localSelectedCards.indexOf(card);
		if (currentIndex < 0) return;
		const nextIndex = currentIndex + direction;
		if (nextIndex < 0 || nextIndex >= localSelectedCards.length) return;
		const nextCards = [...localSelectedCards];
		const [movedCard] = nextCards.splice(currentIndex, 1);
		nextCards.splice(nextIndex, 0, movedCard);
		localSelectedCards = nextCards;
	}

	function startQueueDrag(card: string) {
		draggedQueueCard = card;
	}

	function handleQueueDrop(targetCard: string) {
		if (!draggedQueueCard || draggedQueueCard === targetCard) {
			draggedQueueCard = '';
			return;
		}
		const draggedIndex = localSelectedCards.indexOf(draggedQueueCard);
		const targetIndex = localSelectedCards.indexOf(targetCard);
		if (draggedIndex < 0 || targetIndex < 0) {
			draggedQueueCard = '';
			return;
		}
		const nextCards = [...localSelectedCards];
		const [movedCard] = nextCards.splice(draggedIndex, 1);
		nextCards.splice(targetIndex, 0, movedCard);
		localSelectedCards = nextCards;
		draggedQueueCard = '';
	}

	function submitSelection() {
		if (!canSubmit) return;
		gameServer.submitStellaSelection(localSelectedCards);
		toastStore.trigger({
			message: hasLockedSelection ? '✨ Selections updated' : '✨ Selections locked in',
			autohide: true,
			timeout: 2000
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
	{storytellerLossComplementAuto}
	{votesPerGuesser}
	{votesPerGuesserMin}
	{votesPerGuesserMax}
	{beautyEnabled}
	{beautyVotesPerPlayer}
	{beautyVotesPerPlayerMin}
	{beautyVotesPerPlayerMax}
	{beautyAllowDuplicateVotes}
	{beautyPointsBonus}
	{beautyPointsBonusMin}
	{beautyPointsBonusMax}
	{beautyResultsDisplayMode}
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
	{gameMode}
	showMobileActions={!isObserver}
>
	<svelte:fragment slot="leftRail">
		<div class="card light space-y-2 p-4">
			<h1 class="text-xl font-semibold">Resonance — Associate</h1>
			<p>Clue word: <span class="boujee-text">{clueWord}</span></p>
			<p class="text-xs opacity-75">
				{#if stellaQueueDuringAssociation}
					Pick every matching card, then arrange their reveal queue before locking in.
				{:else}
					Pick every card you want to associate with the clue, then lock in. You can adjust and lock
					again before reveal starts.
				{/if}
			</p>
		</div>
		{#if !isObserver}
			<div class="card light p-4 space-y-3">
				<p class="mb-1 text-sm opacity-80">Draft selected: {localSelectedCards.length}</p>
				<p class="text-xs opacity-70">
					Need {stellaSelectionMin}–{stellaSelectionMax} cards.
				</p>
				{#if hasLockedSelection}
					<p class="mb-2 text-xs opacity-70">
						Locked: {selectedCards.length}{selectionDirty ? ' • draft has changes' : ''}
					</p>
				{/if}
				{#if stellaQueueDuringAssociation && localSelectedCards.length > 0}
					<div class="space-y-2">
						<p class="text-sm font-semibold">Reveal queue</p>
						<div class="space-y-2">
							{#each localSelectedCards as card, index}
								<div
									class="rounded border border-white/20 bg-black/15 p-2"
									role="listitem"
									draggable="true"
									on:dragstart={() => startQueueDrag(card)}
									on:dragover|preventDefault
									on:drop={() => handleQueueDrop(card)}
								>
									<div class="flex items-center justify-between gap-2">
										<div class="min-w-0 text-sm">
											<span class="font-semibold">Q{index + 1}</span>
											<span class="ml-2 opacity-75">Board #{boardIndexByCard[card]}</span>
										</div>
										<div class="flex gap-1">
											<button
												class="btn variant-filled px-2 py-1 text-xs"
												type="button"
												on:click={() => moveQueueCard(card, -1)}
												disabled={index === 0}>↑</button
											>
											<button
												class="btn variant-filled px-2 py-1 text-xs"
												type="button"
												on:click={() => moveQueueCard(card, 1)}
												disabled={index === localSelectedCards.length - 1}>↓</button
											>
										</div>
									</div>
								</div>
							{/each}
						</div>
					</div>
				{/if}
				<button class="btn variant-filled w-full" disabled={!canSubmit} on:click={submitSelection}
					>{submitLabel}</button
				>
			</div>
		{/if}
		{#if isModerator}
			<div class="card light p-4 space-y-2">
				<button class="btn variant-filled w-full" on:click={() => gameServer.resetStellaClue()}
					>Reset clue</button
				>
				<button class="btn variant-filled w-full" on:click={() => gameServer.resetStellaBoard()}
					>Reset board</button
				>
			</div>
		{/if}
	</svelte:fragment>

	<svelte:fragment slot="mobileTop">
		<div class="card light space-y-2 p-4">
			<h1 class="text-xl font-semibold">Resonance — Associate</h1>
			<p>Clue word: <span class="boujee-text">{clueWord}</span></p>
		</div>
	</svelte:fragment>

	<svelte:fragment slot="mobileActions">
		{#if !isObserver}
			<button class="btn variant-filled w-full" disabled={!canSubmit} on:click={submitSelection}
				>{submitLabel}</button
			>
		{/if}
		{#if isModerator}
			<button class="btn variant-filled w-full" on:click={() => gameServer.resetStellaClue()}
				>Reset clue</button
			>
			<button class="btn variant-filled w-full" on:click={() => gameServer.resetStellaBoard()}
				>Reset board</button
			>
		{/if}
	</svelte:fragment>

	<div class="flex h-full flex-col">
		<h2 class="mb-2 hidden text-lg font-semibold lg:block">Shared board</h2>
		{#if !isObserver && stellaQueueDuringAssociation && localSelectedCards.length > 0}
			<div class="mb-3 rounded border border-white/20 bg-black/10 p-3 lg:hidden">
				<p class="mb-2 text-sm font-semibold">Reveal queue</p>
				<div class="space-y-2">
					{#each localSelectedCards as card, index}
						<div
							class="flex items-center justify-between gap-2 rounded border border-white/15 px-2 py-1.5"
						>
							<div class="text-sm">
								<span class="font-semibold">Q{index + 1}</span>
								<span class="ml-2 opacity-75">Board #{boardIndexByCard[card]}</span>
							</div>
							<div class="flex gap-1">
								<button
									class="btn variant-filled px-2 py-1 text-xs"
									type="button"
									on:click={() => moveQueueCard(card, -1)}
									disabled={index === 0}>↑</button
								>
								<button
									class="btn variant-filled px-2 py-1 text-xs"
									type="button"
									on:click={() => moveQueueCard(card, 1)}
									disabled={index === localSelectedCards.length - 1}>↓</button
								>
							</div>
						</div>
					{/each}
				</div>
			</div>
		{/if}
		<div class="min-h-0 flex-1 overflow-y-auto">
			<Images
				{displayImages}
				selectedImages={localSelectedCards}
				selectable={!isObserver}
				desktopFitToHeight={true}
				showIndexOverlay={showVotingCardNumbers}
				imageSecondaryBadges={selectedCardQueueBadges}
				on:select={handleSelect}
			/>
		</div>
	</div>
</StageShell>
