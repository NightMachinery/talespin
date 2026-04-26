<script lang="ts">
	import { getDesktopFitRowCount } from '$lib/cardGrid';
	import { buildBeautyBadgeMetadata } from '$lib/beautyResults';
	import {
		buildCardNumberNavigatorTargetId,
		CARD_NUMBER_NAVIGATOR_SCROLL_MARGIN_TOP
	} from '$lib/cardNumberNavigator';
	import { CARD_IMAGE_ALT_TEXT } from '$lib/cardImageText';
	import CardImage from '$lib/CardImage.svelte';
	import { http_host } from '$lib/gameServer';
	import {
		beautyScoringMode,
		beautyVotePointsDivisorEffective,
		beautyVotePointsDivisorMode
	} from '$lib/mostBeautiful';
	import { cardsFitToHeight } from '$lib/viewOptions';
	import type GameServer from '$lib/gameServer';
	import type { ObserverInfo, PlayerInfo, WinCondition } from '$lib/types';
	import CardNumberNavigator from './CardNumberNavigator.svelte';
	import ChooserNameOverlay from './ChooserNameOverlay.svelte';
	import MyCardsPanel from './MyCardsPanel.svelte';
	import StageShell from './StageShell.svelte';

	export let displayImages: string[] = [];
	export let cardNumberLabels: number[] = [];
	export let name = '';
	export let creator = '';
	export let moderators: string[] = [];
	export let observers: { [key: string]: ObserverInfo } = {};
	export let activePlayer = '';
	export let gameServer: GameServer;
	export let playerToCurrentCards: { [key: string]: string[] } = {};
	export let playerToBeautyVotes: { [key: string]: string[] } = {};
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
	export let beautyVoteTotals: { [key: string]: number } = {};
	export let beautyWinningCards: string[] = [];
	export let roundNum = 0;
	export let cardsRemaining = 0;
	export let deckRefillFlashToken = 0;
	export let winCondition: WinCondition = {
		mode: 'points',
		target_points: 10
	};
	export let myHandImages: string[] = [];
	export let pinnedCards: string[] = [];

	let cardToPlayer: { [key: string]: string } = {};
	let cardToVoterCounts: { [key: string]: { [key: string]: number } } = {};
	let cardToChooserEntries: { [key: string]: { name: string; count?: number }[] } = {};
	let isObserver = false;
	let isModerator = false;
	let canForceStartNextRound = false;
	let viewMode: 'results' | 'hand' = 'results';
	let winningCardSet = new Set<string>();
	let beautyBadges: ReturnType<typeof buildBeautyBadgeMetadata> = {};
	$: formattedEffectiveBeautyDivisor =
		$beautyVotePointsDivisorEffective === null
			? 'pending first beauty results'
			: $beautyVotePointsDivisorEffective.toFixed(1);
	$: isObserver = !!observers[name];
	$: isModerator = new Set(moderators).has(name);
	$: canForceStartNextRound = stage === 'BeautyResults';
	$: resultsDesktopFitEnabled = $cardsFitToHeight;
	$: resultsDesktopFitClass = resultsDesktopFitEnabled ? 'lg:h-full' : '';
	$: resultsDesktopRowCount = getDesktopFitRowCount(displayImages?.length);
	$: resultsSectionClass = resultsDesktopFitEnabled
		? 'results-fit-grid grid w-full grid-cols-2 gap-3 overflow-y-auto lg:min-h-0 lg:flex-1 lg:grid-cols-3 lg:content-stretch'
		: 'grid w-full grid-cols-2 gap-3 overflow-y-auto lg:grid-cols-3 lg:auto-rows-max lg:content-start';
	$: resultsDesktopFitStyle = resultsDesktopFitEnabled
		? `--results-desktop-rows: ${resultsDesktopRowCount};`
		: '';
	$: resultsCardClass = (isWinningCard: boolean) =>
		`${isWinningCard ? 'result-highlight-beauty' : ''} relative overflow-hidden rounded-lg bg-slate-900/35 ${resultsDesktopFitClass}`;
	$: resultsImageClass = `relative w-full object-cover object-center aspect-[2/3] ${resultsDesktopFitClass}`;

	$: {
		cardToPlayer = {};
		cardToVoterCounts = {};
		cardToChooserEntries = {};
		winningCardSet = new Set(beautyWinningCards);
		beautyBadges = buildBeautyBadgeMetadata(beautyVoteTotals);
		Object.entries(playerToCurrentCards).forEach(([key, values]) => {
			for (const value of values || []) {
				cardToPlayer[value] = key;
			}
		});

		Object.entries(playerToBeautyVotes).forEach(([voter, votes]) => {
			for (const votedCard of votes || []) {
				if (!cardToVoterCounts[votedCard]) {
					cardToVoterCounts[votedCard] = {};
				}
				if (!cardToVoterCounts[votedCard][voter]) {
					cardToVoterCounts[votedCard][voter] = 0;
				}
				cardToVoterCounts[votedCard][voter] += 1;
			}
		});

		cardToChooserEntries = Object.fromEntries(
			Object.entries(cardToVoterCounts).map(([card, voterCounts]) => [
				card,
				Object.entries(voterCounts)
					.sort(([a], [b]) => a.localeCompare(b))
					.map(([voter, count]) => ({
						name: voter,
						...(count > 1 ? { count } : {})
					}))
			])
		);
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
>
	<svelte:fragment slot="leftRail">
		<div class="card light space-y-2 p-4">
			<h1 class="text-xl font-semibold">Beauty results</h1>
			<p>
				{#if $beautyScoringMode === 'vote_divisor'}
					Review beauty scoring. Owners gain floor(cumulative current-game beauty votes on their
					submitted cards / K). K mode: {$beautyVotePointsDivisorMode}, effective K:
					{formattedEffectiveBeautyDivisor}.
				{:else}
					Review beauty winners. Each top-voted owner gets +{beautyPointsBonus} once, then continue to
					the next round.
				{/if}
			</p>
		</div>
		<div class="card light p-4">
			<div class="space-y-4">
				<button
					class="btn variant-filled w-full"
					disabled={isObserver}
					on:click={() => gameServer.ready()}>Next Round</button
				>
				{#if isModerator}
					<button
						class="btn variant-filled w-full"
						disabled={!canForceStartNextRound}
						on:click={() => gameServer.forceStartNextRound()}
					>
						Force start next round
					</button>
				{/if}
			</div>
			{#if isObserver}
				<p class="mt-2 text-xs opacity-70">Observers cannot ready up.</p>
			{/if}
		</div>
	</svelte:fragment>

	<svelte:fragment slot="mobileTop">
		<div class="card light space-y-2 p-4">
			<h1 class="text-xl font-semibold">Beauty results</h1>
			<p>
				{#if $beautyScoringMode === 'vote_divisor'}
					Review beauty scoring. Owners gain floor(cumulative current-game beauty votes on their
					submitted cards / K). K mode: {$beautyVotePointsDivisorMode}, effective K:
					{formattedEffectiveBeautyDivisor}.
				{:else}
					Review beauty winners. Each top-voted owner gets +{beautyPointsBonus} once, then continue to
					the next round.
				{/if}
			</p>
		</div>
	</svelte:fragment>

	<svelte:fragment slot="mobileActions">
		<div class="space-y-4">
			<button
				class="btn variant-filled w-full"
				disabled={isObserver}
				on:click={() => gameServer.ready()}>Next Round</button
			>
			{#if isModerator}
				<button
					class="btn variant-filled w-full"
					disabled={!canForceStartNextRound}
					on:click={() => gameServer.forceStartNextRound()}
				>
					Force start next round
				</button>
			{/if}
		</div>
	</svelte:fragment>

	<div class="flex h-full min-h-0 flex-col">
		{#if myHandImages.length > 0}
			<div class="mb-2 grid grid-cols-2 gap-2 lg:max-w-md">
				<button
					type="button"
					class={`btn w-full ${viewMode === 'results' ? 'variant-filled' : 'variant-ghost'}`}
					on:click={() => (viewMode = 'results')}>Results</button
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
			<h2 class="mb-2 hidden text-lg font-semibold lg:block">Beauty cards</h2>
			<CardNumberNavigator
				{cardNumberLabels}
				targetIdScope="beauty-results"
				collapsedLabel="beauty result navigator"
			/>
			<section class={resultsSectionClass} style={resultsDesktopFitStyle}>
				{#each displayImages as image, cardIndex}
					{@const cardNumberLabel = cardNumberLabels[cardIndex] ?? cardIndex + 1}
					<div
						id={buildCardNumberNavigatorTargetId('beauty-results', cardNumberLabel)}
						class={resultsCardClass(winningCardSet.has(image))}
						style:scroll-margin-top={CARD_NUMBER_NAVIGATOR_SCROLL_MARGIN_TOP}
					>
						<CardImage
							src={`${http_host}/cards/${image}`}
							alt={CARD_IMAGE_ALT_TEXT}
							className={resultsImageClass}
						/>
						{#if showVotingCardNumbers}
							<div
								class="absolute left-2 top-2 z-20 rounded bg-black/70 px-2 py-0.5 text-xs font-bold text-white shadow"
							>
								#{cardNumberLabel}
							</div>
						{/if}
						{#if cardToVoterCounts[image]}
							<ChooserNameOverlay
								entries={cardToChooserEntries[image]}
								label={beautyBadges[image]?.label ?? ''}
								labelTier={beautyBadges[image]?.tier ?? 'default'}
								avoidTopLeftBadge={showVotingCardNumbers}
								tone="beauty"
							/>
						{/if}
						<div
							style="bottom: 0;"
							class="rounded-tr w-full absolute bg-primary-200 p-0.5 px-2 text-black font-bold"
						>
							{cardToPlayer[image]}'s card
						</div>
					</div>
				{/each}
			</section>
		{/if}
	</div>
</StageShell>

<style>
	@media (min-width: 1024px) {
		.results-fit-grid {
			grid-template-rows: repeat(var(--results-desktop-rows, 2), minmax(0, 1fr));
		}
	}
</style>
