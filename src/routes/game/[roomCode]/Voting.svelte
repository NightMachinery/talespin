<script lang="ts">
	import { getToastStore } from '@skeletonlabs/skeleton';

	import { getDesktopFitRowCount } from '$lib/cardGrid';
	import {
		buildCardNumberNavigatorTargetId,
		CARD_NUMBER_NAVIGATOR_SCROLL_MARGIN_TOP
	} from '$lib/cardNumberNavigator';
	import { CARD_IMAGE_ALT_TEXT } from '$lib/cardImageText';
	import CardImage from '$lib/CardImage.svelte';
	import { http_host } from '$lib/gameServer';
	import type GameServer from '$lib/gameServer';
	import type { ObserverInfo, PlayerInfo, WinCondition } from '$lib/types';
	import { cardsFitToHeight } from '$lib/viewOptions';
	import CardNumberNavigator from './CardNumberNavigator.svelte';
	import MyCardsPanel from './MyCardsPanel.svelte';
	import StageActionButtons from './StageActionButtons.svelte';
	import StageShell from './StageShell.svelte';

	export let displayImages: string[] = [];
	export let cardNumberLabels: number[] = [];
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
	export let doubleVoteBonusNormalPoints = 1;
	export let doubleVoteBonusTooManyWrongPoints = 1;
	export let doubleVoteBonusTooManyWrongFollowsNormal = true;
	export let doubleVoteBonusTooManyCorrectPoints = 1;
	export let doubleVoteBonusTooManyCorrectFollowsNormal = true;
	export let doubleVoteBonusPointsMin = 0;
	export let doubleVoteBonusPointsMax = 10;
	export let bonusThresholdLossTogglesApplyToAllStorytellerLossRounds = true;
	export let showVotingCardNumbers = true;
	export let roundStartDiscardAllUnpinned = true;
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
	export let disabledCards: string[] = [];
	export let myHandImages: string[] = [];
	export let pinnedCards: string[] = [];

	let selectedVotes: string[] = [];
	let selectedVoteCounts: Record<string, number> = {};
	let cardNumberLabelByImage: Record<string, number> = {};
	let toastStore = getToastStore();
	let isObserver = false;
	let isVoter = false;
	let isModerator = false;
	let canForceRandomVote = false;
	let canAutoObserverify = false;
	let viewMode: 'table' | 'hand' = 'table';
	$: isObserver = !!observers[name];
	$: isVoter = activePlayer !== name && !isObserver;
	$: isModerator = new Set(moderators).has(name);
	$: canForceRandomVote =
		isModerator &&
		Object.entries(players).filter(
			([playerName, info]) => playerName !== activePlayer && info.ready
		).length >= 2;
	$: canAutoObserverify =
		isModerator &&
		Object.entries(players).some(
			([playerName, info]) => playerName !== activePlayer && !info.connected && !info.ready
		);
	$: effectiveVotesPerGuesser = Math.max(
		1,
		Math.min(votesPerGuesser, Math.max(votesPerGuesserMax, 1))
	);
	$: tableDesktopFitEnabled = $cardsFitToHeight;
	$: tableDesktopRowCount = getDesktopFitRowCount(displayImages?.length);
	$: tableSectionClass = tableDesktopFitEnabled
		? 'voting-fit-grid grid w-full grid-cols-2 gap-3 overflow-visible p-1 lg:min-h-0 lg:flex-1 lg:grid-cols-3 lg:content-stretch'
		: 'grid w-full grid-cols-2 gap-3 overflow-visible p-1 lg:grid-cols-3 lg:auto-rows-max lg:content-start';
	$: tableImageClass = tableDesktopFitEnabled
		? 'pointer-events-none aspect-[2/3] w-full rounded-lg object-cover object-center transition-all duration-150 ease-out lg:h-full'
		: 'pointer-events-none aspect-[2/3] w-full rounded-lg object-cover object-center transition-all duration-150 ease-out';
	$: tableButtonClass = tableDesktopFitEnabled
		? 'card-hover-source group relative block w-full overflow-visible rounded-lg bg-slate-900/35 focus-visible:outline-none lg:h-full'
		: 'card-hover-source group relative block w-full overflow-visible rounded-lg bg-slate-900/35 focus-visible:outline-none';
	$: tableDesktopFitStyle = tableDesktopFitEnabled
		? `--voting-desktop-rows: ${tableDesktopRowCount};`
		: '';
	$: canSubmit =
		isVoter && effectiveVotesPerGuesser > 0 && selectedVotes.length === effectiveVotesPerGuesser;
	$: {
		cardNumberLabelByImage = Object.fromEntries(
			displayImages.map((image, index) => [image, cardNumberLabels[index] ?? index + 1])
		);
	}
	$: selectedCardNumberLabels = selectedVotes
		.map((card) => cardNumberLabelByImage[card])
		.filter((label): label is number => typeof label === 'number');
	$: mutedCardNumberLabels = disabledCards
		.map((card) => cardNumberLabelByImage[card])
		.filter((label): label is number => typeof label === 'number');
	$: {
		const disabled = new Set(disabledCards);
		const allowed = new Set(displayImages.filter((image) => !disabled.has(image)));
		const filtered = selectedVotes.filter((card) => allowed.has(card));
		if (filtered.length !== selectedVotes.length) {
			selectedVotes = filtered;
		}
	}
	$: if (selectedVotes.length > effectiveVotesPerGuesser) {
		selectedVotes = selectedVotes.slice(selectedVotes.length - effectiveVotesPerGuesser);
	}
	$: {
		const nextCounts: Record<string, number> = {};
		for (const card of selectedVotes) {
			nextCounts[card] = (nextCounts[card] ?? 0) + 1;
		}
		selectedVoteCounts = nextCounts;
	}

	function voteImageClass(selectedCount: number, isDisabled: boolean) {
		if (isDisabled) {
			return 'cursor-not-allowed ring-[3px] ring-gray-400 saturate-50';
		}
		if (selectedCount >= 2) {
			return 'brightness-110 ring-4 ring-white shadow-xlg double-vote-glow';
		}
		if (selectedCount === 1) {
			return 'brightness-105 ring-4 ring-white shadow-xlg';
		}
		return 'card-hover-target cursor-pointer group-focus-visible:ring-2 group-focus-visible:ring-white/85 group-focus-visible:shadow-[0_0_0_2px_rgba(255,255,255,0.22),0_16px_30px_rgba(0,0,0,0.38)]';
	}

	function cycleCardVote(card: string) {
		if (!isVoter || disabledCards.includes(card)) return;

		const currentCount = selectedVoteCounts[card] ?? 0;
		if (currentCount >= 2) {
			selectedVotes = selectedVotes.filter((value) => value !== card);
			return;
		}
		if (currentCount === 1 && effectiveVotesPerGuesser === 1) {
			selectedVotes = selectedVotes.filter((value) => value !== card);
			return;
		}

		let nextVotes = [...selectedVotes, card];
		while (nextVotes.length > effectiveVotesPerGuesser) {
			nextVotes.shift();
		}
		selectedVotes = nextVotes;
	}

	function submitVotes() {
		if (!canSubmit) {
			toastStore.trigger({
				message: `Use all ${effectiveVotesPerGuesser} vote token${effectiveVotesPerGuesser === 1 ? '' : 's'} before submitting.`,
				autohide: true,
				timeout: 2500
			});
			return;
		}

		gameServer.submitVotes(selectedVotes);
		toastStore.trigger({
			message: '👌 Locked in!',
			autohide: true,
			timeout: 2500
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
	{doubleVoteBonusNormalPoints}
	{doubleVoteBonusTooManyWrongPoints}
	{doubleVoteBonusTooManyWrongFollowsNormal}
	{doubleVoteBonusTooManyCorrectPoints}
	{doubleVoteBonusTooManyCorrectFollowsNormal}
	{doubleVoteBonusPointsMin}
	{doubleVoteBonusPointsMax}
	{bonusThresholdLossTogglesApplyToAllStorytellerLossRounds}
	{showVotingCardNumbers}
	{roundStartDiscardAllUnpinned}
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
	showMobileActions={isVoter || isModerator}
>
	<svelte:fragment slot="leftRail">
		{#if isVoter}
			<div class="card light space-y-2 p-4">
				<h1 class="text-xl font-semibold">
					Which card did <span class="font-bold">{activePlayer}</span> choose for "{description}"?
				</h1>
				<p>Click a card to cycle: single vote → double vote → clear.</p>
				<p class="text-xs opacity-80">
					Votes used: {selectedVotes.length}/{effectiveVotesPerGuesser} (all votes required)
				</p>
			</div>
			<div class="card light p-4">
				<button class="btn variant-filled w-full" disabled={!canSubmit} on:click={submitVotes}
					>Submit Votes</button
				>
			</div>
		{:else}
			<div class="card light space-y-2 p-4">
				<h1 class="text-xl font-semibold">Tallying the votes!</h1>
				<p>Everyone is voting on the clue "{description}".</p>
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
							label: 'Force Random',
							disabled: !canForceRandomVote,
							onClick: () => gameServer.forceCurrentStage()
						},
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
		{#if isVoter}
			<div class="card light space-y-2 p-4">
				<h1 class="text-xl font-semibold">
					Which card did <span class="font-bold">{activePlayer}</span> choose for "{description}"?
				</h1>
				<p>Click a card to cycle: single vote → double vote → clear.</p>
				<p class="text-xs opacity-80">
					Votes used: {selectedVotes.length}/{effectiveVotesPerGuesser} (all votes required)
				</p>
			</div>
		{:else}
			<div class="card light space-y-2 p-4">
				<h1 class="text-xl font-semibold">Tallying the votes!</h1>
				<p>Everyone is voting on the clue "{description}".</p>
				{#if isObserver}
					<p class="opacity-70">You are observing this round.</p>
				{/if}
			</div>
		{/if}
	</svelte:fragment>

	<svelte:fragment slot="mobileActions">
		<div class="space-y-4">
			{#if isVoter}
				<button class="btn variant-filled w-full" disabled={!canSubmit} on:click={submitVotes}
					>Submit Votes</button
				>
			{/if}
			{#if isModerator}
				<StageActionButtons
					actions={[
						{
							label: 'Force Random',
							disabled: !canForceRandomVote,
							onClick: () => gameServer.forceCurrentStage()
						},
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

	<div class="flex h-full min-h-0 flex-col">
		{#if myHandImages.length > 0}
			<div class="mb-2 grid grid-cols-2 gap-2 lg:max-w-md">
				<button
					type="button"
					class={`btn w-full ${viewMode === 'table' ? 'variant-filled' : 'variant-ghost'}`}
					on:click={() => (viewMode = 'table')}>Table</button
				>
				<button
					type="button"
					class={`btn w-full ${viewMode === 'hand' ? 'variant-filled' : 'variant-ghost'}`}
					on:click={() => (viewMode = 'hand')}>My Cards</button
				>
			</div>
		{/if}
		{#if viewMode === 'hand'}
			<MyCardsPanel hand={myHandImages} {pinnedCards} {gameServer} />
		{:else}
			<h2 class="mb-2 hidden text-lg font-semibold lg:block">Cards on table</h2>
			<CardNumberNavigator
				{cardNumberLabels}
				targetIdScope="voting"
				selectedLabels={selectedCardNumberLabels}
				mutedLabels={mutedCardNumberLabels}
				collapsedLabel="voting card navigator"
			/>
			<section class={tableSectionClass} style={tableDesktopFitStyle}>
				{#each displayImages as image, cardIndex}
					{@const cardNumberLabel = cardNumberLabels[cardIndex] ?? cardIndex + 1}
					{@const selectedCount = selectedVoteCounts[image] ?? 0}
					{@const isDisabled = disabledCards.includes(image)}
					<button
						type="button"
						id={buildCardNumberNavigatorTargetId('voting', cardNumberLabel)}
						class={`${tableButtonClass} ${isDisabled || !isVoter ? 'cursor-default' : ''}`}
						style:scroll-margin-top={CARD_NUMBER_NAVIGATOR_SCROLL_MARGIN_TOP}
						disabled={!isVoter || isDisabled}
						on:click={() => cycleCardVote(image)}
					>
						<CardImage
							className={`${tableImageClass} ${voteImageClass(selectedCount, isDisabled)}`}
							src={`${http_host}/cards/${image}`}
							alt={CARD_IMAGE_ALT_TEXT}
						/>
						{#if showVotingCardNumbers}
							<div
								class="absolute right-2 top-2 z-20 rounded bg-black/70 px-2 py-0.5 text-xs font-bold text-white shadow"
							>
								#{cardNumberLabel}
							</div>
						{/if}
						{#if selectedCount > 0}
							<div
								class={`absolute left-2 top-2 z-20 rounded px-2 py-0.5 text-xs font-bold text-white ${
									selectedCount >= 2
										? 'bg-success-500 shadow-[0_0_0_2px_rgba(255,255,255,0.3)]'
										: 'bg-primary-500 shadow-[0_0_0_1px_rgba(255,255,255,0.25)]'
								}`}
							>
								×{selectedCount}
							</div>
						{/if}
					</button>
				{/each}
			</section>
		{/if}
	</div>
</StageShell>

<style>
	.double-vote-glow {
		box-shadow:
			0 0 0 2px rgb(255 255 255 / 0.65),
			0 0 0 8px rgb(var(--color-primary-500) / 0.45),
			0 0 28px rgb(var(--color-primary-500) / 0.75),
			0 22px 44px rgb(0 0 0 / 0.55);
	}

	@media (min-width: 1024px) {
		.voting-fit-grid {
			grid-template-rows: repeat(var(--voting-desktop-rows, 2), minmax(0, 1fr));
		}
	}
</style>
