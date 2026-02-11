<script lang="ts">
	import type GameServer from '$lib/gameServer';
	import type { ObserverInfo, PlayerInfo, WinCondition } from '$lib/types';
	import Images from './Images.svelte';
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
	export let storytellerLossThreshold = 1;
	export let storytellerLossThresholdMin = 1;
	export let storytellerLossThresholdMax = 1;
	export let votesPerGuesser = 1;
	export let votesPerGuesserMin = 1;
	export let votesPerGuesserMax = 1;
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
	$: isObserver = !!observers[name];
	$: isActivePlayer = activePlayer === name && !isObserver;

	function activePlayerChoose() {
		gameServer.activePlayerChoose(selectedImage, descriptionBox);
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
	{storytellerLossThreshold}
	{storytellerLossThresholdMin}
	{storytellerLossThresholdMax}
	{votesPerGuesser}
	{votesPerGuesserMin}
	{votesPerGuesserMax}
	{pointChange}
	{activePlayer}
	{roundNum}
	{cardsRemaining}
	{deckRefillFlashToken}
	{winCondition}
	showMobileActions={isActivePlayer}
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
	</svelte:fragment>

	<div class="flex h-full flex-col">
		<h2 class="mb-2 hidden text-lg font-semibold lg:block">{name}, your six cards</h2>
		<div class="min-h-0 flex-1 overflow-y-auto">
			<Images {displayImages} bind:selectedImage selectable={isActivePlayer} mode="hand" />
		</div>
	</div>
</StageShell>
