<script lang="ts">
	import type GameServer from '$lib/gameServer';
	import type { ObserverInfo, PlayerInfo, WinCondition } from '$lib/types';
	import Images from './Images.svelte';
	import StageShell from './StageShell.svelte';
	import { getToastStore } from '@skeletonlabs/skeleton';

	export let displayImages: string[] = [];
	export let name = '';
	export let creator = '';
	export let moderators: string[] = [];
	export let observers: { [key: string]: ObserverInfo } = {};
	export let activePlayer = '';
	export let gameServer: GameServer;
	export let description = '';
	export let players: { [key: string]: PlayerInfo } = {};
	export let allowNewPlayersMidgame = true;
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
	let selectedImage = '';
	let isObserver = false;
	let isChooser = false;
	$: isObserver = !!observers[name];
	$: isChooser = activePlayer !== name && !isObserver;

	if (name !== activePlayer && !isObserver) {
		toastStore.trigger({
			message: '👉 Your turn!',
			autohide: true,
			timeout: 5000
		});
	}

	function choose() {
		gameServer.playersChoose(selectedImage);
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
	{pointChange}
	{activePlayer}
	{roundNum}
	{cardsRemaining}
	{deckRefillFlashToken}
	{winCondition}
	showMobileActions={isChooser}
>
	<svelte:fragment slot="leftRail">
		{#if isChooser}
			<div class="card light space-y-2 p-4">
				<h1 class="text-xl font-semibold">Your turn!</h1>
				<p>
					Choose a card that <span class="boujee-text">{activePlayer}</span> would put for "{description}"
				</p>
			</div>
			<div class="card light p-4">
				<button class="btn variant-filled w-full" disabled={selectedImage === ''} on:click={choose}
					>Choose</button
				>
			</div>
		{:else}
			<div class="card light space-y-2 p-4">
				<h1 class="text-2xl">Sit tight!</h1>
				<p>Players are choosing cards that match "{description}"</p>
				{#if isObserver}
					<p class="opacity-70">You are observing this round.</p>
				{/if}
			</div>
		{/if}
	</svelte:fragment>

	<svelte:fragment slot="mobileTop">
		{#if isChooser}
			<div class="card light space-y-2 p-4">
				<h1 class="text-xl font-semibold">Your turn!</h1>
				<p>
					Choose a card that <span class="boujee-text">{activePlayer}</span> would put for "{description}"
				</p>
			</div>
		{:else}
			<div class="card light space-y-2 p-4">
				<h1 class="text-2xl">Sit tight!</h1>
				<p>Players are choosing cards that match "{description}"</p>
				{#if isObserver}
					<p class="opacity-70">You are observing this round.</p>
				{/if}
			</div>
		{/if}
	</svelte:fragment>

	<svelte:fragment slot="mobileActions">
		{#if isChooser}
			<button class="btn variant-filled w-full" disabled={selectedImage === ''} on:click={choose}
				>Choose</button
			>
		{/if}
	</svelte:fragment>

	<div class="flex h-full flex-col">
		<h2 class="mb-2 hidden text-lg font-semibold lg:block">Your hand</h2>
		<div class="min-h-0 flex-1 overflow-y-auto">
			<Images {displayImages} bind:selectedImage selectable={isChooser} mode="hand" />
		</div>
	</div>
</StageShell>
