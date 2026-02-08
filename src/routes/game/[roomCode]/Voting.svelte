<script lang="ts">
	import { Accordion, AccordionItem } from '@skeletonlabs/skeleton';
	import { getToastStore } from '@skeletonlabs/skeleton';

	import type GameServer from '$lib/gameServer';
	import type { PlayerInfo } from '$lib/types';
	import Images from './Images.svelte';
	import StageShell from './StageShell.svelte';

	export let displayImages: string[] = [];
	export let name = '';
	export let activePlayer = '';
	export let gameServer: GameServer;
	export let description = '';
	export let players: { [key: string]: PlayerInfo } = {};
	export let stage = '';
	export let pointChange: { [key: string]: number } = {};
	export let roundNum = 0;

	let selectedImage = '';
	let toastStore = getToastStore();
	let isVoter = false;
	$: isVoter = activePlayer !== name;

	function vote() {
		gameServer.vote(selectedImage);
		toastStore.trigger({
			message: '👌 Locked in!',
			autohide: true,
			timeout: 2500
		});
	}
</script>

<StageShell {players} {stage} {pointChange} {activePlayer} {roundNum} showMobileActions={isVoter}>
	<svelte:fragment slot="leftRail">
		<div class="card light p-4">
			<h2 class="mb-2 text-lg font-semibold">How points work</h2>
			<Accordion>
				<AccordionItem>
					<svelte:fragment slot="summary">
						<strong
							>If all players find <span class="boujee-text">{activePlayer}'s</span> card:</strong
						>
					</svelte:fragment>
					<svelte:fragment slot="content">
						<ul class="ml-8">
							<li>+0 for <span class="font-bold">{activePlayer}</span></li>
							<li>+2 for everyone else</li>
						</ul>
					</svelte:fragment>
				</AccordionItem>
				<AccordionItem>
					<svelte:fragment slot="summary">
						<strong
							>If nobody finds <span class="boujee-text">{activePlayer}'s</span> card:</strong
						>
					</svelte:fragment>
					<svelte:fragment slot="content">
						<ul class="ml-8">
							<li>+0 for <span class="font-bold">{activePlayer}</span></li>
							<li>+2 for everyone else</li>
							<li>+1 bonus point for each vote your card receives</li>
						</ul>
					</svelte:fragment>
				</AccordionItem>
				<AccordionItem>
					<svelte:fragment slot="summary">
						<strong>Otherwise</strong>
					</svelte:fragment>
					<svelte:fragment slot="content">
						<ul class="ml-8">
							<li>+3 for <span class="font-bold">{activePlayer}</span></li>
							<li>+3 for you if you find <span>{activePlayer}'s</span> card</li>
							<li>+1 bonus point for each vote your card receives</li>
						</ul>
					</svelte:fragment>
				</AccordionItem>
			</Accordion>
		</div>

		{#if isVoter}
			<div class="card light space-y-2 p-4">
				<h1 class="text-xl font-semibold">
					Which card did <span class="font-bold">{activePlayer}</span> choose for "{description}"?
				</h1>
				<p>Don't fall for the fakes!</p>
			</div>
			<div class="card light p-4">
				<button class="btn variant-filled w-full" disabled={selectedImage === ''} on:click={vote}
					>Vote</button
				>
			</div>
		{:else}
			<div class="card light space-y-2 p-4">
				<h1 class="text-xl font-semibold">Tallying the votes!</h1>
				<p>Everyone is voting on the clue "{description}".</p>
			</div>
		{/if}
	</svelte:fragment>

	<svelte:fragment slot="mobileTop">
		{#if isVoter}
			<div class="card light space-y-2 p-4">
				<h1 class="text-xl font-semibold">
					Which card did <span class="font-bold">{activePlayer}</span> choose for "{description}"?
				</h1>
				<p>Don't fall for the fakes!</p>
			</div>
		{:else}
			<div class="card light space-y-2 p-4">
				<h1 class="text-xl font-semibold">Tallying the votes!</h1>
				<p>Everyone is voting on the clue "{description}".</p>
			</div>
		{/if}
	</svelte:fragment>

	<svelte:fragment slot="mobileBottom">
		<div class="card light p-4">
			<h2 class="mb-2 text-lg font-semibold">How points work</h2>
			<Accordion>
				<AccordionItem>
					<svelte:fragment slot="summary">
						<strong
							>If all players find <span class="boujee-text">{activePlayer}'s</span> card:</strong
						>
					</svelte:fragment>
					<svelte:fragment slot="content">
						<ul class="ml-8">
							<li>+0 for <span class="font-bold">{activePlayer}</span></li>
							<li>+2 for everyone else</li>
						</ul>
					</svelte:fragment>
				</AccordionItem>
				<AccordionItem>
					<svelte:fragment slot="summary">
						<strong
							>If nobody finds <span class="boujee-text">{activePlayer}'s</span> card:</strong
						>
					</svelte:fragment>
					<svelte:fragment slot="content">
						<ul class="ml-8">
							<li>+0 for <span class="font-bold">{activePlayer}</span></li>
							<li>+2 for everyone else</li>
							<li>+1 bonus point for each vote your card receives</li>
						</ul>
					</svelte:fragment>
				</AccordionItem>
				<AccordionItem>
					<svelte:fragment slot="summary">
						<strong>Otherwise</strong>
					</svelte:fragment>
					<svelte:fragment slot="content">
						<ul class="ml-8">
							<li>+3 for <span class="font-bold">{activePlayer}</span></li>
							<li>+3 for you if you find <span>{activePlayer}'s</span> card</li>
							<li>+1 bonus point for each vote your card receives</li>
						</ul>
					</svelte:fragment>
				</AccordionItem>
			</Accordion>
		</div>
	</svelte:fragment>

	<svelte:fragment slot="mobileActions">
		{#if isVoter}
			<button class="btn variant-filled w-full" disabled={selectedImage === ''} on:click={vote}
				>Vote</button
			>
		{/if}
	</svelte:fragment>

	<div class="flex h-full flex-col">
		<h2 class="mb-2 hidden text-lg font-semibold lg:block">Cards on table</h2>
		<div class="min-h-0 flex-1 overflow-y-auto">
			<Images {displayImages} bind:selectedImage selectable={isVoter} mode="board" />
		</div>
	</div>
</StageShell>
