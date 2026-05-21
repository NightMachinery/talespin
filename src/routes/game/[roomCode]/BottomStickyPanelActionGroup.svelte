<script lang="ts">
	import {
		FastForwardIcon,
		PlayIcon,
		RefreshCwIcon,
		RotateCcwIcon,
		ShuffleIcon,
		SkipForwardIcon,
		UserXIcon
	} from 'svelte-feather-icons';
	import {
		BOTTOM_STICKY_PANEL_ACTION_LAYOUT,
		bottomStickyPanelActionTooltip,
		bottomStickyPanelGridStyle,
		bottomStickyPanelVisibleLabel,
		type BottomStickyPanelAction,
		type BottomStickyPanelActionIcon,
		type BottomStickyPanelActionLayout
	} from '$lib/bottomStickyPanel';

	export let actions: BottomStickyPanelAction[] = [];
	export let layout: BottomStickyPanelActionLayout = BOTTOM_STICKY_PANEL_ACTION_LAYOUT;

	const longPressDelayMs = 520;
	let visibleTooltip = '';
	let longPressTimer: ReturnType<typeof setTimeout> | null = null;

	$: gridStyle = bottomStickyPanelGridStyle(actions.length);

	function showTooltip(action: BottomStickyPanelAction) {
		visibleTooltip = bottomStickyPanelActionTooltip(action);
	}

	function hideTooltip() {
		visibleTooltip = '';
	}

	function clearLongPressTimer() {
		if (!longPressTimer) return;
		clearTimeout(longPressTimer);
		longPressTimer = null;
	}

	function handlePointerDown(action: BottomStickyPanelAction) {
		clearLongPressTimer();
		longPressTimer = setTimeout(() => showTooltip(action), longPressDelayMs);
	}

	function handlePointerEnd() {
		clearLongPressTimer();
		hideTooltip();
	}

	function runAction(action: BottomStickyPanelAction) {
		hideTooltip();
		if (
			action.confirmMessage &&
			typeof window !== 'undefined' &&
			!window.confirm(action.confirmMessage)
		) {
			return;
		}
		action.onClick();
	}

	function iconLabel(icon: BottomStickyPanelActionIcon) {
		return icon;
	}
</script>

{#if layout === 'row'}
	<div class="relative">
		{#if visibleTooltip}
			<div
				class="pointer-events-none absolute bottom-[calc(100%+0.5rem)] left-1/2 z-30 max-w-[min(18rem,calc(100vw-2rem))] -translate-x-1/2 rounded bg-surface-900 px-3 py-1.5 text-center text-xs font-semibold text-white shadow-lg"
				role="tooltip"
			>
				{visibleTooltip}
			</div>
		{/if}
		<div class="grid gap-2" style={gridStyle}>
			{#each actions as action (action.label)}
				{@const visibleLabel = bottomStickyPanelVisibleLabel(action)}
				{@const tooltip = bottomStickyPanelActionTooltip(action)}
				<button
					type="button"
					class="btn variant-filled relative min-h-10 w-full min-w-0 gap-1 whitespace-normal px-1.5 text-xs leading-tight sm:text-sm"
					disabled={action.disabled ?? false}
					aria-label={action.label}
					title={tooltip}
					on:pointerdown={() => handlePointerDown(action)}
					on:pointerup={handlePointerEnd}
					on:pointerleave={handlePointerEnd}
					on:pointercancel={handlePointerEnd}
					on:mouseenter={() => showTooltip(action)}
					on:mouseleave={hideTooltip}
					on:focus={() => showTooltip(action)}
					on:blur={hideTooltip}
					on:contextmenu|preventDefault
					on:click={() => runAction(action)}
				>
					{#if action.icon === 'rotate-ccw'}
						<span aria-hidden="true"><RotateCcwIcon size="16" /></span>
					{:else if action.icon === 'refresh-cw'}
						<span aria-hidden="true"><RefreshCwIcon size="16" /></span>
					{:else if action.icon === 'shuffle'}
						<span aria-hidden="true"><ShuffleIcon size="16" /></span>
					{:else if action.icon === 'skip-forward'}
						<span aria-hidden="true"><SkipForwardIcon size="16" /></span>
					{:else if action.icon === 'fast-forward'}
						<span aria-hidden="true"><FastForwardIcon size="16" /></span>
					{:else if action.icon === 'user-x'}
						<span aria-hidden="true"><UserXIcon size="16" /></span>
					{:else if action.icon === 'play'}
						<span aria-hidden="true"><PlayIcon size="16" /></span>
					{/if}
					{#if visibleLabel}
						<span class="min-w-0 break-words">{visibleLabel}</span>
					{:else if action.icon}
						<span class="sr-only">{iconLabel(action.icon)}</span>
					{/if}
				</button>
			{/each}
		</div>
	</div>
{:else}
	<div class="space-y-2">
		{#each actions as action (action.label)}
			<button
				type="button"
				class="btn variant-filled w-full"
				disabled={action.disabled ?? false}
				on:click={() => runAction(action)}
			>
				{action.label}
			</button>
		{/each}
	</div>
{/if}
