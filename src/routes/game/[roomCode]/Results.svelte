<script lang="ts">
	import { http_host } from '$lib/gameServer';
	import type GameServer from '$lib/gameServer';
	import type { PlayerInfo } from '$lib/types';
	import StageShell from './StageShell.svelte';

	export let displayImages: string[] = [];
	export let activeCard = '';
	export let activePlayer = '';
	export let gameServer: GameServer;
	export let playerToCurrentCard: { [key: string]: string } = {};
	export let playerToVote: { [key: string]: string } = {};
	export let players: { [key: string]: PlayerInfo } = {};
	export let stage = '';
	export let pointChange: { [key: string]: number } = {};
	export let roundNum = 0;

	let cardToPlayer: { [key: string]: string } = {};
	let cardToVoters: { [key: string]: string[] } = {};

	$: {
		cardToPlayer = {};
		cardToVoters = {};
		Object.entries(playerToCurrentCard).forEach(([key, value]) => {
			cardToPlayer[value] = key;
		});

		Object.entries(playerToVote).forEach(([key, value]) => {
			if (!cardToVoters[value]) {
				cardToVoters[value] = [];
			}
			cardToVoters[value].push(key);
		});
	}
</script>

<StageShell {players} {stage} {pointChange} {activePlayer} {roundNum}>
	<svelte:fragment slot="leftRail">
		<div class="card light space-y-2 p-4">
			<h1 class="text-xl font-semibold">Round complete</h1>
			<p>Review votes and scores, then continue to the next round.</p>
		</div>
		<div class="card light p-4">
			<button class="btn variant-filled w-full" on:click={() => gameServer.ready()}>Next Round</button>
		</div>
	</svelte:fragment>

	<svelte:fragment slot="mobileTop">
		<div class="card light space-y-2 p-4">
			<h1 class="text-xl font-semibold">Round complete</h1>
			<p>Review votes and scores, then continue to the next round.</p>
		</div>
	</svelte:fragment>

	<svelte:fragment slot="mobileActions">
		<button class="btn variant-filled w-full" on:click={() => gameServer.ready()}>Next Round</button>
	</svelte:fragment>

	<div class="flex h-full flex-col">
		<h2 class="mb-2 hidden text-lg font-semibold lg:block">Round cards</h2>
		<section class="grid h-full w-full grid-cols-2 gap-3 overflow-y-auto lg:grid-cols-3">
			{#each displayImages as image}
				<div
					class={`${activeCard == image ? 'boujee-border' : ''} relative h-full overflow-hidden rounded-lg`}
				>
					<img
						src={`${http_host}/cards/${image}`}
						alt="You can't play this game without the images!"
						class="relative h-full w-full object-cover object-center aspect-[2/3]"
					/>
					{#if cardToVoters[image]}
						<div class="absolute" style="top: 20px; right: 12px;">
							<div class="flex flex-col gap-2">
								{#each cardToVoters[image] as voter}
									<div class="bg-success-500 px-1.5 rounded text-black font-bold">
										🔘 {voter}
									</div>
								{/each}
							</div>
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
</style>
