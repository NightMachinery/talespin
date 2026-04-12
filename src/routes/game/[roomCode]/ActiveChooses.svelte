<script lang="ts">
	import type GameServer from '$lib/gameServer';
	import type { ObserverInfo, PlayerInfo, WinCondition } from '$lib/types';
	import Images from './Images.svelte';
	import StageActionButtons from './StageActionButtons.svelte';
	import StageShell from './StageShell.svelte';
	import { getToastStore } from '@skeletonlabs/skeleton';

	export let displayImages: string[] = [];
	export let activePlayer = '';
	export let name = '';
	export let creator = '';
	export let moderators: string[] = [];
	export let observers: { [key: string]: ObserverInfo } = {};
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

	let toastStore = getToastStore();
	let descriptionBox = '';
	let selectedImage = '';
	let isObserver = false;
	let isActivePlayer = false;
	let isModerator = false;
	let canForceSwitchStoryteller = false;
	let lastActivePlayer = '';
	$: isObserver = !!observers[name];
	$: isActivePlayer = activePlayer === name && !isObserver;
	$: isModerator = new Set(moderators).has(name);
	$: canForceSwitchStoryteller = isModerator && Object.keys(players).length >= 2;
	$: if (activePlayer !== lastActivePlayer) {
		lastActivePlayer = activePlayer;
		descriptionBox = '';
		selectedImage = '';
	}

	function activePlayerChoose() {
		gameServer.activePlayerChoose(selectedImage, descriptionBox);
	}

	function handleCardSelect(event: CustomEvent<string>) {
		selectedImage = event.detail;
	}

	if (activePlayer === name && !isObserver) {
		toastStore.trigger({
			message: '👉 Your turn!',
			autohide: true,
			timeout: 5000
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
	{beautySplitPointsOnTie}
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
	showMobileActions={isActivePlayer || isModerator}
>
	<svelte:fragment slot="leftRail">
		{#if isActivePlayer}
			<div class="card light space-y-2 p-4">
				<h1 class="text-xl font-semibold">Choose a card and write a one-word description</h1>
				<p class="opacity-80">Pick your card and clue, then lock it in.</p>
			</div>
			<div class="card light space-y-3 p-4">
				<label class="text-sm font-semibold" for="description-desktop">Description</label>
				<input
					id="description-desktop"
					type="text"
					placeholder="Description"
					bind:value={descriptionBox}
					class="w-full rounded border px-3 py-2 text-gray-700 shadow"
				/>
				<button
					class="btn variant-filled w-full"
					disabled={selectedImage === '' || descriptionBox === ''}
					on:click={activePlayerChoose}>Choose</button
				>
			</div>
		{:else}
			<div class="card light space-y-2 p-4">
				<h1 class="text-2xl">Sit tight!</h1>
				<p>
					Waiting for <span class="boujee-text">{activePlayer}</span> to choose a card and description
				</p>
				{#if isObserver}
					<p class="opacity-70">You are observing this round.</p>
				{/if}
			</div>
		{/if}
		{#if isModerator}
			<div class="card light p-4">
				<StageActionButtons
					actions={[
						{
							label: 'Switch Storyteller',
							disabled: !canForceSwitchStoryteller,
							onClick: () => gameServer.forceCurrentStage()
						}
					]}
				/>
			</div>
		{/if}
	</svelte:fragment>

	<svelte:fragment slot="mobileTop">
		{#if isActivePlayer}
			<div class="card light space-y-2 p-4">
				<h1 class="text-xl font-semibold">Choose a card and write a one-word description</h1>
				<p class="opacity-80">Pick your card and clue, then lock it in.</p>
			</div>
		{:else}
			<div class="card light space-y-2 p-4">
				<h1 class="text-2xl">Sit tight!</h1>
				<p>
					Waiting for <span class="boujee-text">{activePlayer}</span> to choose a card and description
				</p>
				{#if isObserver}
					<p class="opacity-70">You are observing this round.</p>
				{/if}
			</div>
		{/if}
	</svelte:fragment>

	<svelte:fragment slot="mobileActions">
		{#if isActivePlayer}
			<div class="space-y-3">
				<input
					id="description-mobile"
					type="text"
					placeholder="Description"
					bind:value={descriptionBox}
					class="w-full rounded border px-3 py-2 text-gray-700 shadow"
				/>
				<button
					class="btn variant-filled w-full"
					disabled={selectedImage === '' || descriptionBox === ''}
					on:click={activePlayerChoose}>Choose</button
				>
			</div>
		{/if}
		{#if isModerator}
			<StageActionButtons
				actions={[
					{
						label: 'Switch Storyteller',
						disabled: !canForceSwitchStoryteller,
						onClick: () => gameServer.forceCurrentStage()
					}
				]}
			/>
		{/if}
	</svelte:fragment>

	<div class="flex h-full flex-col">
		<h2 class="mb-2 hidden text-lg font-semibold lg:block">{name}, your cards</h2>
		<div class="min-h-0 flex-1 overflow-y-auto">
			<Images
				{displayImages}
				selectedImages={selectedImage ? [selectedImage] : []}
				selectable={isActivePlayer}
				mode="hand"
				on:select={handleCardSelect}
			/>
		</div>
	</div>
</StageShell>
