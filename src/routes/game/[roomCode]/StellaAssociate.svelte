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
	const toastStore = getToastStore();
	$: isObserver = !!observers[name];
	$: isModerator = new Set(moderators).has(name);
	$: canSubmit = !isObserver && localSelectedCards.length > 0;
	$: hasLockedSelection = selectedCards.length > 0;
	$: selectionDirty = !sameSelections(localSelectedCards, selectedCards);
	$: submitLabel = hasLockedSelection ? 'Update locked selections' : 'Lock selections';
	$: roundKey = `${roundNum}::${clueWord}::${displayImages.join('||')}`;
	$: selectedKey = selectedCards.join('||');
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
				Pick every card you want to associate with the clue, then lock in. You can adjust and lock
				again before reveal starts.
			</p>
		</div>
		{#if !isObserver}
			<div class="card light p-4">
				<p class="mb-1 text-sm opacity-80">Draft selected: {localSelectedCards.length}</p>
				{#if hasLockedSelection}
					<p class="mb-2 text-xs opacity-70">
						Locked: {selectedCards.length}{selectionDirty ? ' • draft has changes' : ''}
					</p>
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
	</svelte:fragment>

	<div class="flex h-full flex-col">
		<h2 class="mb-2 hidden text-lg font-semibold lg:block">Shared board</h2>
		<div class="min-h-0 flex-1 overflow-y-auto">
			<Images
				{displayImages}
				selectedImages={localSelectedCards}
				selectable={!isObserver}
				desktopFitToHeight={true}
				showIndexOverlay={showVotingCardNumbers}
				on:select={handleSelect}
			/>
		</div>
	</div>
</StageShell>
