<script lang="ts">
	import { getDesktopFitRowCount } from '$lib/cardGrid';
	import { CARD_IMAGE_ALT_TEXT } from '$lib/cardImageText';
	import CardImage from '$lib/CardImage.svelte';
	import { http_host } from '$lib/gameServer';
	import { cardsFitToHeight } from '$lib/viewOptions';
	import type GameServer from '$lib/gameServer';
	import type { ObserverInfo, PlayerInfo, WinCondition } from '$lib/types';
	import ChooserNameOverlay from './ChooserNameOverlay.svelte';
	import StageShell from './StageShell.svelte';

	export let displayImages: string[] = [];
	export let name = '';
	export let creator = '';
	export let moderators: string[] = [];
	export let observers: { [key: string]: ObserverInfo } = {};
	export let activeCard = '';
	export let activePlayer = '';
	export let gameServer: GameServer;
	export let playerToCurrentCards: { [key: string]: string[] } = {};
	export let playerToVotes: { [key: string]: string[] } = {};
	export let playerToBeautyVotes: { [key: string]: string[] } = {};
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

	let cardToPlayer: { [key: string]: string } = {};
	let cardToVoterCounts: { [key: string]: { [key: string]: number } } = {};
	let cardToChooserEntries: { [key: string]: { name: string; count?: number }[] } = {};
	let beautyCardToVoterCounts: { [key: string]: { [key: string]: number } } = {};
	let beautyCardToChooserEntries: { [key: string]: { name: string; count?: number }[] } = {};
	let beautyWinningCardSet = new Set<string>();
	let isObserver = false;
	let isModerator = false;
	let canForceStartNextRound = false;
	$: isObserver = !!observers[name];
	$: isModerator = new Set(moderators).has(name);
	$: canForceStartNextRound = stage === 'Results';
	$: resultsDesktopFitEnabled = $cardsFitToHeight;
	$: resultsDesktopFitClass = resultsDesktopFitEnabled ? 'lg:h-full' : '';
	$: resultsDesktopRowCount = getDesktopFitRowCount(displayImages?.length);
	$: resultsSectionClass = resultsDesktopFitEnabled
		? 'results-fit-grid grid w-full grid-cols-2 gap-3 overflow-y-auto lg:min-h-0 lg:flex-1 lg:grid-cols-3 lg:content-stretch'
		: 'grid w-full grid-cols-2 gap-3 overflow-y-auto lg:grid-cols-3 lg:auto-rows-max lg:content-start';
	$: resultsDesktopFitStyle = resultsDesktopFitEnabled
		? `--results-desktop-rows: ${resultsDesktopRowCount};`
		: '';
	$: resultsCardClass = (isActiveCard: boolean) =>
		`${isActiveCard ? 'boujee-border' : ''} relative overflow-hidden rounded-lg bg-slate-900/35 ${resultsDesktopFitClass}`;
	$: resultsImageClass = `relative w-full object-cover object-center aspect-[2/3] ${resultsDesktopFitClass}`;

	$: {
		cardToPlayer = {};
		cardToVoterCounts = {};
		cardToChooserEntries = {};
		beautyCardToVoterCounts = {};
		beautyCardToChooserEntries = {};
		beautyWinningCardSet = new Set(beautyWinningCards);
		Object.entries(playerToCurrentCards).forEach(([key, values]) => {
			for (const value of values || []) {
				cardToPlayer[value] = key;
			}
		});

		Object.entries(playerToVotes).forEach(([voter, votes]) => {
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

		Object.entries(playerToBeautyVotes).forEach(([voter, votes]) => {
			for (const votedCard of votes || []) {
				if (!beautyCardToVoterCounts[votedCard]) {
					beautyCardToVoterCounts[votedCard] = {};
				}
				if (!beautyCardToVoterCounts[votedCard][voter]) {
					beautyCardToVoterCounts[votedCard][voter] = 0;
				}
				beautyCardToVoterCounts[votedCard][voter] += 1;
			}
		});

		beautyCardToChooserEntries = Object.fromEntries(
			Object.entries(beautyCardToVoterCounts).map(([card, voterCounts]) => [
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
			<h1 class="text-xl font-semibold">Round complete</h1>
			<p>
				{#if beautyEnabled && beautyResultsDisplayMode === 'separate'}
					Review storyteller votes and scores, then continue to beauty results.
				{:else}
					Review votes and scores, then continue to the next round.
				{/if}
			</p>
		</div>
		<div class="card light p-4">
			<div class="space-y-2">
				<button
					class="btn variant-filled w-full"
					disabled={isObserver}
					on:click={() => gameServer.ready()}
				>
					{beautyEnabled && beautyResultsDisplayMode === 'separate'
						? 'Beauty Results'
						: 'Next Round'}
				</button>
				{#if isModerator}
					<button
						class="btn variant-filled w-full"
						disabled={!canForceStartNextRound}
						on:click={() => gameServer.forceStartNextRound()}
					>
						{beautyEnabled && beautyResultsDisplayMode === 'separate'
							? 'Force beauty results'
							: 'Force start next round'}
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
			<h1 class="text-xl font-semibold">Round complete</h1>
			<p>
				{#if beautyEnabled && beautyResultsDisplayMode === 'separate'}
					Review storyteller votes and scores, then continue to beauty results.
				{:else}
					Review votes and scores, then continue to the next round.
				{/if}
			</p>
		</div>
	</svelte:fragment>

	<svelte:fragment slot="mobileActions">
		<div class="space-y-2">
			<button
				class="btn variant-filled w-full"
				disabled={isObserver}
				on:click={() => gameServer.ready()}
			>
				{beautyEnabled && beautyResultsDisplayMode === 'separate' ? 'Beauty Results' : 'Next Round'}
			</button>
			{#if isModerator}
				<button
					class="btn variant-filled w-full"
					disabled={!canForceStartNextRound}
					on:click={() => gameServer.forceStartNextRound()}
				>
					{beautyEnabled && beautyResultsDisplayMode === 'separate'
						? 'Force beauty results'
						: 'Force start next round'}
				</button>
			{/if}
		</div>
	</svelte:fragment>

	<div class="flex h-full min-h-0 flex-col">
		<h2 class="mb-2 hidden text-lg font-semibold lg:block">Round cards</h2>
		<section class={resultsSectionClass} style={resultsDesktopFitStyle}>
			{#each displayImages as image, cardIndex}
				<div class={resultsCardClass(activeCard == image)}>
					<CardImage
						src={`${http_host}/cards/${image}`}
						alt={CARD_IMAGE_ALT_TEXT}
						className={resultsImageClass}
					/>
					{#if showVotingCardNumbers}
						<div
							class="absolute left-2 top-2 z-20 rounded bg-black/70 px-2 py-0.5 text-xs font-bold text-white shadow"
						>
							#{cardIndex + 1}
						</div>
					{/if}
					{#if cardToVoterCounts[image]}
						<ChooserNameOverlay
							entries={cardToChooserEntries[image]}
							label={beautyEnabled && beautyResultsDisplayMode === 'combined' ? 'Guess' : ''}
							avoidTopLeftBadge={showVotingCardNumbers}
						/>
					{/if}
					{#if beautyEnabled && beautyResultsDisplayMode === 'combined' && beautyCardToVoterCounts[image]}
						<ChooserNameOverlay
							entries={beautyCardToChooserEntries[image]}
							label="Beauty"
							position="bottom-right"
						/>
					{:else if beautyEnabled && beautyResultsDisplayMode === 'summary' && (typeof beautyVoteTotals[image] === 'number' || beautyWinningCardSet.has(image))}
						<div
							class="absolute bottom-8 left-2 z-20 rounded bg-fuchsia-200 px-2 py-0.5 text-xs font-bold text-black shadow"
						>
							Beauty: {beautyVoteTotals[image] ?? 0}
						</div>
					{/if}
					{#if beautyEnabled && beautyWinningCardSet.has(image)}
						<div
							class={`absolute bottom-8 z-20 rounded bg-success-300 px-2 py-0.5 text-xs font-bold text-black shadow ${
								beautyResultsDisplayMode === 'combined' ? 'left-2' : 'right-2'
							}`}
						>
							Winner
						</div>
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
	</div>
</StageShell>

<style>
	@property --bg-angle {
		inherits: false;
		initial-value: 0deg;
		syntax: '<angle>';
	}
	.boujee-border {
		animation: spin 2.5s infinite linear;
		background:
			linear-gradient(to bottom, rgb(var(--color-primary-500)), rgb(var(--color-primary-500)))
				padding-box,
			conic-gradient(from var(--bg-angle) in oklch longer hue, rgb(var(--color-success-500)) 0 0)
				border-box;
		border: 5px solid transparent;
		box-shadow: 0.125rem 0.25rem 0.25rem 0.5rem oklch(0.1 0.37 315 / 0.25);
	}

	@keyframes spin {
		to {
			--bg-angle: 360deg;
		}
	}

	@media (min-width: 1024px) {
		.results-fit-grid {
			grid-template-rows: repeat(var(--results-desktop-rows, 2), minmax(0, 1fr));
		}
	}
</style>
