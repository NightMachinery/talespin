<script lang="ts">
	import type GameServer from '$lib/gameServer';
	import Images from './Images.svelte';

	export let hand: string[] = [];
	export let pinnedCards: string[] = [];
	export let gameServer: GameServer;
	export let title = 'My Cards';
	export let canTogglePins = true;

	$: pinnedSet = new Set(pinnedCards);

	function handlePinToggle(event: CustomEvent<string>) {
		if (!canTogglePins) return;
		const card = event.detail;
		gameServer.setHandCardPinned(card, !pinnedSet.has(card));
	}
</script>

<div class="flex h-full min-h-0 flex-col">
	<h2 class="mb-2 hidden text-lg font-semibold lg:block">{title}</h2>
	<div class="min-h-0 flex-1 overflow-y-auto">
		{#if hand.length > 0}
			<Images
				displayImages={hand}
				mode="hand"
				selectable={canTogglePins}
				pinnedImages={pinnedCards}
				showPinBadges
				pinTogglesEnabled={canTogglePins}
				on:select={handlePinToggle}
				on:pinToggle={handlePinToggle}
			/>
		{:else}
			<div class="card light p-4 text-center opacity-80">No cards in your hand right now.</div>
		{/if}
	</div>
</div>
