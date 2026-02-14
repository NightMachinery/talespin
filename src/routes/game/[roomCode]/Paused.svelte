<script lang="ts">
	import type GameServer from '$lib/gameServer';
	import type { ObserverInfo, PlayerInfo, WinCondition } from '$lib/types';
	import StageShell from './StageShell.svelte';

	export let name = '';
	export let creator = '';
	export let moderators: string[] = [];
	export let players: { [key: string]: PlayerInfo } = {};
	export let observers: { [key: string]: ObserverInfo } = {};
	export let gameServer: GameServer;
	export let reason = '';
	export let allowNewPlayersMidgame = true;
	export let storytellerLossComplement = 0;
	export let storytellerLossComplementMin = 0;
	export let storytellerLossComplementMax = 0;
	export let votesPerGuesser = 1;
	export let votesPerGuesserMin = 1;
	export let votesPerGuesserMax = 1;
	export let cardsPerHand = 6;
	export let cardsPerHandMin = 1;
	export let cardsPerHandMax = 12;
	export let nominationsPerGuesser = 1;
	export let nominationsPerGuesserMin = 1;
	export let nominationsPerGuesserMax = 1;
	export let bonusCorrectGuessOnThresholdCorrectLoss = false;
	export let bonusDoubleVoteOnThresholdCorrectLoss = false;
	export let showVotingCardNumbers = false;
	export let roundStartDiscardCount = 0;
	export let roundNum = 0;
	export let cardsRemaining = 0;
	export let deckRefillFlashToken = 0;
	export let winCondition: WinCondition = {
		mode: 'points',
		target_points: 10
	};

	$: moderatorSet = new Set(moderators);
	$: isModerator = moderatorSet.has(name);
	$: nonObserverPlayerCount = Object.keys(players).length;
	$: canResume = nonObserverPlayerCount >= 3;
</script>

<StageShell
	{players}
	{observers}
	{name}
	{creator}
	{moderators}
	{gameServer}
	stage="Paused"
	{allowNewPlayersMidgame}
	{storytellerLossComplement}
	{storytellerLossComplementMin}
	{storytellerLossComplementMax}
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
	activePlayer=""
	pointChange={{}}
	{roundNum}
	{cardsRemaining}
	{deckRefillFlashToken}
	{winCondition}
	showMobileActions={isModerator}
>
	<svelte:fragment slot="leftRail">
		<div class="card light space-y-2 p-4">
			<h1 class="text-xl font-semibold">Game paused</h1>
			<p>{reason || 'Need at least 3 non-observer players to continue.'}</p>
		</div>
		{#if isModerator}
			<div class="card light p-4">
				<button
					class="btn variant-filled w-full"
					disabled={!canResume}
					on:click={() => gameServer.resumeGame()}
				>
					Resume Game
				</button>
				{#if !canResume}
					<p class="mt-2 text-xs opacity-70">Need at least 3 non-observer players to resume.</p>
				{/if}
			</div>
		{/if}
	</svelte:fragment>

	<svelte:fragment slot="mobileTop">
		<div class="card light space-y-2 p-4">
			<h1 class="text-xl font-semibold">Game paused</h1>
			<p>{reason || 'Need at least 3 non-observer players to continue.'}</p>
		</div>
	</svelte:fragment>

	<svelte:fragment slot="mobileActions">
		{#if isModerator}
			<button
				class="btn variant-filled w-full"
				disabled={!canResume}
				on:click={() => gameServer.resumeGame()}
			>
				Resume Game
			</button>
		{/if}
	</svelte:fragment>

	<div class="card light p-6">
		<p>Waiting for enough non-observer players.</p>
	</div>
</StageShell>
