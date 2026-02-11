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
	export let storytellerLossThreshold = 1;
	export let storytellerLossThresholdMin = 1;
	export let storytellerLossThresholdMax = 1;
	export let votesPerGuesser = 1;
	export let votesPerGuesserMin = 1;
	export let votesPerGuesserMax = 1;
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
	{storytellerLossThreshold}
	{storytellerLossThresholdMin}
	{storytellerLossThresholdMax}
	{votesPerGuesser}
	{votesPerGuesserMin}
	{votesPerGuesserMax}
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
