<script lang="ts">
	import { getToastStore } from '@skeletonlabs/skeleton';
	import type GameServer from '$lib/gameServer';
	import type { ObserverInfo, PlayerInfo, WinCondition } from '$lib/types';
	import StageActionButtons from './StageActionButtons.svelte';
	import StageShell from './StageShell.svelte';

	export let name = '';
	export let creator = '';
	export let moderators: string[] = [];
	export let observers: { [key: string]: ObserverInfo } = {};
	export let activePlayer = '';
	export let gameServer: GameServer;
	export let description = '';
	export let players: { [key: string]: PlayerInfo } = {};
	export let serverTimeMs: number | null = null;
	export let currentStageDeadlineS: number | null = null;
	export let stage = 'ClueRating';
	export let pointChange: { [key: string]: number } = {};
	export let roundNum = 0;
	export let cardsRemaining = 0;
	export let deckRefillFlashToken = 0;
	export let maxStars = 3;
	export let winCondition: WinCondition = {
		mode: 'points',
		target_points: 10
	};

	const toastStore = getToastStore();
	let selectedStars = 0;
	let hoverStars = 0;

	$: isObserver = !!observers[name];
	$: isStoryteller = activePlayer === name;
	$: canRate = !isObserver && !isStoryteller;
	$: isModerator = new Set(moderators).has(name);
	$: canAutoObserverify =
		isModerator &&
		Object.entries(players).some(
			([playerName, info]) => playerName !== activePlayer && !info.connected && !info.ready
		);

	function pickStars(stars: number) {
		if (!canRate) return;
		selectedStars = stars;
	}

	function submitRating() {
		if (!canRate || selectedStars < 1) {
			toastStore.trigger({
				message: 'Pick a star rating first.',
				autohide: true,
				timeout: 2200
			});
			return;
		}

		gameServer.submitClueRating(selectedStars);
		toastStore.trigger({
			message: `🌟 Locked in ${selectedStars}/${maxStars}.`,
			autohide: true,
			timeout: 2200
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
	{activePlayer}
	{serverTimeMs}
	{currentStageDeadlineS}
	{pointChange}
	{roundNum}
	{cardsRemaining}
	{deckRefillFlashToken}
	{winCondition}
>
	<svelte:fragment slot="mobileTop">
		<div class="card clue-rating-shell p-4">
			<p class="text-xs font-semibold uppercase tracking-[0.3em] text-amber-200/80">Clue Rating</p>
			<h2 class="mt-2 text-xl font-semibold text-white">{description}</h2>
			<p class="mt-2 text-sm text-white/75">
				Rate {activePlayer}&rsquo;s clue before results land.
			</p>
		</div>
	</svelte:fragment>

	<svelte:fragment slot="leftRail">
		<div class="card clue-rating-shell p-4">
			<p class="text-xs font-semibold uppercase tracking-[0.3em] text-amber-200/80">Clue Rating</p>
			<h2 class="mt-2 text-2xl font-semibold text-white">{description}</h2>
			<p class="mt-2 text-sm text-white/75">Storyteller: {activePlayer}</p>
		</div>
	</svelte:fragment>

	<div class="flex h-full flex-col justify-center">
		<div class="clue-rating-shell mx-auto w-full max-w-4xl rounded-[2rem] p-6 sm:p-8">
			<div class="mx-auto max-w-3xl text-center">
				<p class="text-xs font-semibold uppercase tracking-[0.35em] text-amber-200/80">
					How radiant was that clue?
				</p>
				<h1 class="mt-4 text-3xl font-black text-white sm:text-5xl">{description}</h1>
				<p class="mx-auto mt-4 max-w-2xl text-sm text-white/75 sm:text-base">
					Give the storyteller a little constellation of approval. Missing votes are skipped.
				</p>
			</div>

			{#if canRate}
				<div class="mt-8 flex flex-wrap items-center justify-center gap-3 sm:gap-4">
					{#each Array.from({ length: maxStars }, (_, index) => index + 1) as starValue}
						<button
							type="button"
							class={`star-button ${(hoverStars || selectedStars) >= starValue ? 'active' : ''}`}
							aria-label={`${starValue} star${starValue === 1 ? '' : 's'}`}
							on:mouseenter={() => (hoverStars = starValue)}
							on:mouseleave={() => (hoverStars = 0)}
							on:focus={() => (hoverStars = starValue)}
							on:blur={() => (hoverStars = 0)}
							on:click={() => pickStars(starValue)}
						>
							<span class="star-core">★</span>
							<span class="star-label">{starValue}</span>
						</button>
					{/each}
				</div>

				<div class="mt-6 text-center">
					<p class="text-sm text-white/75">
						{#if selectedStars > 0}
							You picked <span class="font-semibold text-amber-200">{selectedStars}</span> /
							{maxStars}.
						{:else}
							Choose from 1 to {maxStars} stars.
						{/if}
					</p>
				</div>

				<div class="mx-auto mt-8 max-w-sm">
					<button
						class="btn clue-submit-btn w-full"
						disabled={selectedStars < 1}
						on:click={submitRating}
					>
						Submit star rating
					</button>
				</div>
			{:else}
				<div class="mx-auto mt-10 max-w-2xl text-center">
					<div class="rounded-3xl border border-white/15 bg-white/5 px-6 py-8 backdrop-blur-sm">
						<p class="text-6xl">✨</p>
						<h2 class="mt-4 text-2xl font-semibold text-white">
							{#if isStoryteller}
								Your clue is being rated
							{:else}
								Observers are waiting for ratings
							{/if}
						</h2>
						<p class="mt-3 text-white/75">
							{#if isStoryteller}
								Sit back while the table crowns your clue with stars.
							{:else}
								Non-storytellers are casting stars before the reveal.
							{/if}
						</p>
					</div>
				</div>
			{/if}
		</div>
	</div>

	<svelte:fragment slot="mobileActions">
		{#if canRate}
			<button
				class="btn clue-submit-btn w-full"
				disabled={selectedStars < 1}
				on:click={submitRating}
			>
				Submit {selectedStars > 0 ? `${selectedStars}★` : 'rating'}
			</button>
		{:else if isModerator}
			<StageActionButtons
				actions={[
					{ label: 'Force Skip', onClick: () => gameServer.forceCurrentStage() },
					{
						label: 'Auto-observerify offline',
						disabled: !canAutoObserverify,
						onClick: () => gameServer.autoObserverifyOfflinePendingPlayers()
					}
				]}
			/>
		{/if}
	</svelte:fragment>

	<svelte:fragment slot="sidebarBottom">
		{#if isModerator}
			<StageActionButtons
				actions={[
					{ label: 'Force Skip', onClick: () => gameServer.forceCurrentStage() },
					{
						label: 'Auto-observerify offline',
						disabled: !canAutoObserverify,
						onClick: () => gameServer.autoObserverifyOfflinePendingPlayers()
					}
				]}
			/>
		{/if}
	</svelte:fragment>
</StageShell>

<style>
	.clue-rating-shell {
		position: relative;
		overflow: hidden;
		background: radial-gradient(circle at 20% 20%, rgb(251 191 36 / 0.2), transparent 35%),
			radial-gradient(circle at 80% 15%, rgb(125 211 252 / 0.18), transparent 30%),
			radial-gradient(circle at 50% 85%, rgb(192 132 252 / 0.2), transparent 30%),
			linear-gradient(145deg, rgb(15 23 42 / 0.95), rgb(30 41 59 / 0.92));
		border: 1px solid rgb(255 255 255 / 0.14);
		box-shadow:
			0 24px 80px rgb(15 23 42 / 0.45),
			inset 0 1px 0 rgb(255 255 255 / 0.12);
	}

	.star-button {
		position: relative;
		display: flex;
		height: 5.5rem;
		width: 5.5rem;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		border-radius: 9999px;
		border: 1px solid rgb(255 255 255 / 0.14);
		background: rgb(255 255 255 / 0.05);
		color: rgb(226 232 240 / 0.92);
		transition:
			transform 180ms ease,
			box-shadow 180ms ease,
			background 180ms ease,
			border-color 180ms ease;
	}

	.star-button:hover,
	.star-button:focus-visible,
	.star-button.active {
		transform: translateY(-3px) scale(1.03);
		border-color: rgb(253 230 138 / 0.65);
		background: linear-gradient(180deg, rgb(251 191 36 / 0.28), rgb(250 204 21 / 0.12));
		box-shadow:
			0 14px 36px rgb(251 191 36 / 0.25),
			0 0 24px rgb(251 191 36 / 0.15);
		color: white;
	}

	.star-core {
		font-size: 2rem;
		line-height: 1;
	}

	.star-label {
		margin-top: 0.15rem;
		font-size: 0.8rem;
		font-weight: 700;
		opacity: 0.88;
	}

	.clue-submit-btn {
		background: linear-gradient(135deg, rgb(250 204 21), rgb(245 158 11)),
			linear-gradient(135deg, rgb(255 255 255 / 0.16), rgb(255 255 255 / 0));
		color: rgb(17 24 39);
		font-weight: 800;
		box-shadow: 0 18px 40px rgb(245 158 11 / 0.25);
	}

	@media (prefers-reduced-motion: reduce) {
		.star-button {
			transition: none;
		}
	}
</style>
