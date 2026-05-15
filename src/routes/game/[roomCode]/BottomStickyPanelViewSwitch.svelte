<script lang="ts">
	import {
		BOTTOM_STICKY_PANEL_VIEW_PRESENTATION,
		bottomStickyPanelGridStyle,
		type BottomStickyPanelViewPresentation
	} from '$lib/bottomStickyPanel';
	import ViewModeToggle from './ViewModeToggle.svelte';

	export let value: string;
	export let modes: { id: string; label: string; icon: 'table' | 'hand' | 'results' }[] = [];
	export let presentation: BottomStickyPanelViewPresentation =
		BOTTOM_STICKY_PANEL_VIEW_PRESENTATION;

	$: gridStyle = bottomStickyPanelGridStyle(modes.length);
</script>

{#if presentation === 'icon'}
	<ViewModeToggle bind:value {modes} />
{:else}
	<div class="grid gap-2" style={gridStyle}>
		{#each modes as mode}
			<button
				type="button"
				class={`btn w-full min-w-0 whitespace-normal px-2 text-sm leading-tight ${
					value === mode.id ? 'variant-filled' : 'variant-ghost'
				}`}
				aria-label={mode.label}
				title={mode.label}
				on:click={() => (value = mode.id)}
			>
				{mode.label}
			</button>
		{/each}
	</div>
{/if}
