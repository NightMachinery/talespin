<script lang="ts">
	import {
		BOTTOM_STICKY_PANEL_ACTION_LAYOUT,
		bottomStickyPanelGridStyle,
		type BottomStickyPanelActionLayout
	} from '$lib/bottomStickyPanel';

	export let actions: { label: string; disabled?: boolean; onClick: () => void }[] = [];
	export let layout: BottomStickyPanelActionLayout = BOTTOM_STICKY_PANEL_ACTION_LAYOUT;

	$: gridStyle = bottomStickyPanelGridStyle(actions.length);
</script>

{#if layout === 'row'}
	<div class="grid gap-2" style={gridStyle}>
		{#each actions as action (action.label)}
			<button
				type="button"
				class="btn variant-filled min-h-10 w-full min-w-0 whitespace-normal px-2 text-xs leading-tight sm:text-sm"
				disabled={action.disabled ?? false}
				on:click={action.onClick}
			>
				{action.label}
			</button>
		{/each}
	</div>
{:else}
	<div class="space-y-3">
		{#each actions as action (action.label)}
			<button
				type="button"
				class="btn variant-filled w-full"
				disabled={action.disabled ?? false}
				on:click={action.onClick}
			>
				{action.label}
			</button>
		{/each}
	</div>
{/if}
