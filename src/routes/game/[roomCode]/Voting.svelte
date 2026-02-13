<script lang="ts">
	import { getToastStore } from '@skeletonlabs/skeleton';

	import { CARD_IMAGE_ALT_TEXT } from '$lib/cardImageText';
	import { http_host } from '$lib/gameServer';
	import type GameServer from '$lib/gameServer';
	import type { ObserverInfo, PlayerInfo, WinCondition } from '$lib/types';
	import StageShell from './StageShell.svelte';

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
	export let stage = '';
	export let pointChange: { [key: string]: number } = {};
	export let roundNum = 0;
	export let cardsRemaining = 0;
	export let deckRefillFlashToken = 0;
	export let winCondition: WinCondition = {
		mode: 'points',
		target_points: 10
	};
	export let disabledCards: string[] = [];

	let selectedVotes: string[] = [];
	let toastStore = getToastStore();
	let isObserver = false;
	let isVoter = false;
	$: isObserver = !!observers[name];
	$: isVoter = activePlayer !== name && !isObserver;
	$: guesserCount = Math.max(0, Object.keys(players).length - 1);
	$: effectiveLossThreshold = Math.max(0, guesserCount - storytellerLossComplement);
	$: effectiveVotesPerGuesser = Math.max(
		1,
		Math.min(votesPerGuesser, Math.max(votesPerGuesserMax, 1))
	);
	$: canSubmit =
		isVoter && effectiveVotesPerGuesser > 0 && selectedVotes.length === effectiveVotesPerGuesser;
	$: {
		const disabled = new Set(disabledCards);
		const allowed = new Set(displayImages.filter((image) => !disabled.has(image)));
		const filtered = selectedVotes.filter((card) => allowed.has(card));
		if (filtered.length !== selectedVotes.length) {
			selectedVotes = filtered;
		}
	}
	$: if (selectedVotes.length > effectiveVotesPerGuesser) {
		selectedVotes = selectedVotes.slice(selectedVotes.length - effectiveVotesPerGuesser);
	}

	function voteCountForCard(card: string) {
		return selectedVotes.filter((value) => value === card).length;
	}

	function cycleCardVote(card: string) {
		if (!isVoter || disabledCards.includes(card)) return;

		const currentCount = voteCountForCard(card);
		if (currentCount >= 2) {
			selectedVotes = selectedVotes.filter((value) => value !== card);
			return;
		}
		if (currentCount === 1 && effectiveVotesPerGuesser === 1) {
			selectedVotes = selectedVotes.filter((value) => value !== card);
			return;
		}

		let nextVotes = [...selectedVotes, card];
		while (nextVotes.length > effectiveVotesPerGuesser) {
			nextVotes.shift();
		}
		selectedVotes = nextVotes;
	}

	function submitVotes() {
		if (!canSubmit) {
			toastStore.trigger({
				message: `Use all ${effectiveVotesPerGuesser} vote token${effectiveVotesPerGuesser === 1 ? '' : 's'} before submitting.`,
				autohide: true,
				timeout: 2500
			});
			return;
		}

		gameServer.submitVotes(selectedVotes);
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
	{pointChange}
	{activePlayer}
	{roundNum}
	{cardsRemaining}
	{deckRefillFlashToken}
	{winCondition}
	showMobileActions={isVoter}
>
	<svelte:fragment slot="leftRail">
		{#if isVoter}
			<div class="card light space-y-2 p-4">
				<h1 class="text-xl font-semibold">
					Which card did <span class="font-bold">{activePlayer}</span> choose for "{description}"?
				</h1>
				<p>Click a card to cycle: single vote → double vote → clear.</p>
				<p class="text-xs opacity-80">
					Votes used: {selectedVotes.length}/{effectiveVotesPerGuesser} (all votes required)
				</p>
			</div>
			<div class="card light p-4">
				<button class="btn variant-filled w-full" disabled={!canSubmit} on:click={submitVotes}
					>Submit Votes</button
				>
			</div>
		{:else}
			<div class="card light space-y-2 p-4">
				<h1 class="text-xl font-semibold">Tallying the votes!</h1>
				<p>Everyone is voting on the clue "{description}".</p>
				{#if isObserver}
					<p class="opacity-70">You are observing this round.</p>
				{/if}
			</div>
		{/if}
	</svelte:fragment>

	<svelte:fragment slot="mobileTop">
		{#if isVoter}
			<div class="card light space-y-2 p-4">
				<h1 class="text-xl font-semibold">
					Which card did <span class="font-bold">{activePlayer}</span> choose for "{description}"?
				</h1>
				<p>Click a card to cycle: single vote → double vote → clear.</p>
				<p class="text-xs opacity-80">
					Votes used: {selectedVotes.length}/{effectiveVotesPerGuesser} (all votes required)
				</p>
			</div>
		{:else}
			<div class="card light space-y-2 p-4">
				<h1 class="text-xl font-semibold">Tallying the votes!</h1>
				<p>Everyone is voting on the clue "{description}".</p>
				{#if isObserver}
					<p class="opacity-70">You are observing this round.</p>
				{/if}
			</div>
		{/if}
	</svelte:fragment>

	<svelte:fragment slot="sidebarBottom">
		<div class="card light p-4">
			<h2 class="mb-2 text-lg font-semibold">How points work</h2>
			<p class="mb-2 text-xs opacity-80">
				There are <span class="font-semibold">{guesserCount}</span> guessers.
				With C=<span class="font-semibold">{storytellerLossComplement}</span>, storyteller-loss threshold is
				<span class="font-semibold">{effectiveLossThreshold}</span>.
			</p>
			<ul class="ml-5 list-disc space-y-1 text-sm">
				<li>
					Loss scenario triggers when right guesses ≥ {effectiveLossThreshold} or wrong guesses ≥
					{effectiveLossThreshold}.
				</li>
				<li>Loss scenario points: <span class="font-semibold">{activePlayer}</span> +0, each guesser +2.</li>
				<li>
					Otherwise points: <span class="font-semibold">{activePlayer}</span> +3, each guesser with at least
					one correct vote +3.
				</li>
				<li>
					Double-correct bonus: +1 if 2+ of your {effectiveVotesPerGuesser} vote
					{effectiveVotesPerGuesser === 1 ? '' : 's'} hit storyteller card.
				</li>
				<li>Decoy bonus: +1 per vote token on your card (max +3, non-storyteller only).</li>
			</ul>
		</div>
	</svelte:fragment>

	<svelte:fragment slot="mobileActions">
		{#if isVoter}
			<button class="btn variant-filled w-full" disabled={!canSubmit} on:click={submitVotes}
				>Submit Votes</button
			>
		{/if}
	</svelte:fragment>

	<div class="flex h-full flex-col">
		<h2 class="mb-2 hidden text-lg font-semibold lg:block">Cards on table</h2>
		<section class="grid w-full grid-cols-2 gap-3 overflow-visible p-1 lg:grid-cols-3 lg:auto-rows-max lg:content-start">
			{#each displayImages as image}
				{@const selectedCount = voteCountForCard(image)}
				{@const isDisabled = disabledCards.includes(image)}
				<button
					type="button"
					class={`group relative block w-full overflow-visible rounded-lg focus-visible:outline-none ${isDisabled || !isVoter ? 'cursor-default' : ''}`}
					disabled={!isVoter || isDisabled}
					on:click={() => cycleCardVote(image)}
				>
					<img
						class={`pointer-events-none w-full rounded-lg object-cover object-center aspect-[2/3] transition-all duration-150 ease-out ${
							isDisabled
								? 'cursor-not-allowed ring-[3px] ring-gray-400'
								: selectedCount >= 2
									? 'brightness-110 ring-4 ring-white shadow-xlg'
									: selectedCount === 1
										? 'brightness-105 ring-2 ring-white shadow-xlg'
									: 'cursor-pointer lg:group-hover:ring-2 lg:group-hover:ring-white/85 lg:group-hover:brightness-105'
						}`}
						src={`${http_host}/cards/${image}`}
						alt={CARD_IMAGE_ALT_TEXT}
					/>
					{#if selectedCount === 1}
						<div class="pointer-events-none absolute inset-2 rounded-md border-2 border-white/90"></div>
					{:else if selectedCount >= 2}
						<div class="pointer-events-none absolute inset-2 rounded-md border-4 border-white/90"></div>
						<div class="pointer-events-none absolute inset-5 rounded-md border-2 border-white/95"></div>
					{/if}
					{#if selectedCount > 0}
						<div
							class={`absolute left-2 top-2 rounded px-2 py-0.5 text-xs font-bold text-white ${
								selectedCount >= 2 ? 'bg-success-500' : 'bg-primary-500'
							}`}
						>
							×{selectedCount}
						</div>
					{/if}
				</button>
			{/each}
		</section>
	</div>
</StageShell>
