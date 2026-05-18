<script lang="ts">
	import type {
		BottomStickyPanelAction,
		BottomStickyPanelActionLayout
	} from '$lib/bottomStickyPanel';
	import BottomStickyPanelActionGroup from './BottomStickyPanelActionGroup.svelte';

	export let actions: BottomStickyPanelAction[] = [];
	export let layout: BottomStickyPanelActionLayout = 'stack';
</script>

<BottomStickyPanelActionGroup {actions} {layout} />
