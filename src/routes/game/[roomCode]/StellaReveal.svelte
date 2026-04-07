<script lang="ts">
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
	export let cardPoints: { [key: string]: number } = {};
	export let darkPlayer = '';
	export let gameMode: GameMode = 'stella';

	$: isObserver = !!observers[name];
	$: isScout = activePlayer === name && !isObserver;
	$: revealableCards = selectedCards.filter((card) => !revealedCards.includes(card));

	function reveal(card: string) {
		if (!isScout) return;
		gameServer.revealStellaCard(card);
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
>
	<svelte:fragment slot="leftRail">
		<div class="card light space-y-2 p-4">
			<h1 class="text-xl font-semibold">Stella — Reveal</h1>
			<p>Clue word: <span class="boujee-text">{clueWord}</span></p>
			<p>Scout: <span class="font-semibold">{activePlayer}</span></p>
			{#if darkPlayer}
				<p class="text-warning-200">In the Dark: {darkPlayer}</p>
			{/if}
		</div>
		<div class="card light space-y-2 p-4">
			<h2 class="font-semibold">Selection counts</h2>
			{#each Object.entries(selectedCounts).sort( ([a], [b]) => a.localeCompare(b) ) as [playerName, count]}
				<p>{playerName}: {count}</p>
			{/each}
		</div>
		{#if isScout}
			<div class="card light space-y-2 p-4">
				<h2 class="font-semibold">Reveal one of your unrevealed picks</h2>
				{#each revealableCards as card}
					<button class="btn variant-filled w-full" on:click={() => reveal(card)}
						>Reveal {card}</button
					>
				{/each}
			</div>
		{/if}
	</svelte:fragment>

	<svelte:fragment slot="mobileTop">
		<div class="card light space-y-2 p-4">
			<h1 class="text-xl font-semibold">Stella — Reveal</h1>
			<p>{clueWord}</p>
			<p>Scout: {activePlayer}</p>
		</div>
	</svelte:fragment>

	<div class="flex h-full flex-col">
		<h2 class="mb-2 hidden text-lg font-semibold lg:block">Shared board</h2>
		<div class="min-h-0 flex-1 overflow-y-auto">
			<Images {displayImages} selectedImages={revealedCards} selectable={false} />
		</div>
		<div class="mt-3 grid grid-cols-1 gap-2 md:grid-cols-2 xl:grid-cols-3">
			{#each displayImages as image}
				{#if cardPoints[image] !== undefined}
					<div class="card light p-2 text-sm">
						<div class="font-semibold">{image}</div>
						<div>
							{cardPoints[image] === 0
								? 'Fall'
								: `Worth ${cardPoints[image]} star${cardPoints[image] === 1 ? '' : 's'}`}
						</div>
					</div>
				{/if}
			{/each}
		</div>
	</div>
</StageShell>
