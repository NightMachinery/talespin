<script lang="ts">
	import { transparentCardNameOverlays } from '$lib/viewOptions';

	type ChooserOverlayEntry = {
		name: string;
		count?: number;
	};

	export let entries: ChooserOverlayEntry[] = [];
	export let avoidTopLeftBadge = false;

	$: entryCount = entries.length;
	$: usesTwoColumns = entryCount >= 4;
	$: hasLongEntry = entries.some((entry) => entry.name.length >= 10);
	$: textSizeClass =
		entryCount >= 7 || hasLongEntry ? 'text-[10px]' : usesTwoColumns ? 'text-[11px]' : 'text-xs';
	$: overlayGridClass = usesTwoColumns ? 'grid-cols-2' : 'grid-cols-1';
	$: chipClass = $transparentCardNameOverlays
		? 'border border-white/55 bg-black/10 text-white'
		: 'bg-black/75 text-white';
	$: containerClass = avoidTopLeftBadge
		? 'pointer-events-none absolute left-14 right-2 top-2 z-20'
		: 'pointer-events-none absolute right-2 top-2 z-20 max-w-[calc(100%-1rem)]';
</script>

{#if entries.length > 0}
	<div class={containerClass}>
		<div class={`grid ${overlayGridClass} gap-1`}>
			{#each entries as entry}
				<div
					class={`min-w-0 truncate rounded px-1.5 py-0.5 font-bold leading-tight shadow ${textSizeClass} ${chipClass}`}
				>
					{entry.name}{entry.count && entry.count > 1 ? ` ×${entry.count}` : ''}
				</div>
			{/each}
		</div>
	</div>
{/if}
