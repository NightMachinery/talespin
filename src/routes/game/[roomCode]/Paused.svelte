<script lang="ts">
	import type GameServer from '$lib/gameServer';
	import type { ObserverInfo, PlayerInfo } from '$lib/types';
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

	$: moderatorSet = new Set(moderators);
	$: isModerator = moderatorSet.has(name);
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
	activePlayer=""
	pointChange={{}}
	roundNum={0}
	cardsRemaining={0}
	deckRefillFlashToken={0}
	showMobileActions={isModerator}
>
	<svelte:fragment slot="leftRail">
		<div class="card light space-y-2 p-4">
			<h1 class="text-xl font-semibold">Game paused</h1>
			<p>{reason || 'Need at least 3 non-observer players to continue.'}</p>
		</div>
		{#if isModerator}
			<div class="card light p-4">
				<button class="btn variant-filled w-full" on:click={() => gameServer.resumeGame()}>
					Resume Game
				</button>
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
			<button class="btn variant-filled w-full" on:click={() => gameServer.resumeGame()}>
				Resume Game
			</button>
		{/if}
	</svelte:fragment>

	<div class="card light p-6">
		<p>Waiting for enough non-observer players.</p>
	</div>
</StageShell>
