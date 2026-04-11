<script lang="ts">
	import { transparentCardNameOverlays } from '$lib/viewOptions';

	type ChooserOverlayEntry = {
		name: string;
		count?: number;
	};

	export let entries: ChooserOverlayEntry[] = [];
	export let avoidTopLeftBadge = false;
	export let label = '';
	export let position: 'top-right' | 'bottom-right' | 'bottom-left' = 'top-right';
	export let tone: 'story' | 'beauty' = 'story';

	$: entryCount = entries.length;
	$: usesTwoColumns = entryCount >= 4;
	$: hasLongEntry = entries.some((entry) => entry.name.length >= 10);
	$: textSizeClass =
		entryCount >= 7 || hasLongEntry ? 'text-[10px]' : usesTwoColumns ? 'text-[11px]' : 'text-xs';
	$: overlayGridClass = usesTwoColumns ? 'grid-cols-2' : 'grid-cols-1';
	$: baseChipClass =
		tone === 'beauty'
			? $transparentCardNameOverlays
				? 'border border-fuchsia-200/70 bg-fuchsia-500/15 text-fuchsia-50'
				: 'bg-fuchsia-700/85 text-fuchsia-50'
			: $transparentCardNameOverlays
				? 'border border-sky-100/70 bg-slate-900/15 text-sky-50'
				: 'bg-slate-900/80 text-sky-50';
	$: labelChipClass =
		tone === 'beauty'
			? $transparentCardNameOverlays
				? 'border border-rose-200/75 bg-rose-500/20 text-rose-50'
				: 'bg-rose-500/90 text-black'
			: $transparentCardNameOverlays
				? 'border border-sky-100/75 bg-sky-500/20 text-sky-50'
				: 'bg-sky-400/90 text-black';
	$: containerClass =
		position === 'bottom-right'
			? 'pointer-events-none absolute bottom-10 right-2 z-20 max-w-[calc(100%-1rem)]'
			: position === 'bottom-left'
				? 'pointer-events-none absolute bottom-10 left-2 z-20 max-w-[calc(100%-1rem)]'
				: avoidTopLeftBadge
					? 'pointer-events-none absolute left-2 right-2 top-10 z-20'
					: 'pointer-events-none absolute right-2 top-2 z-20 max-w-[calc(100%-1rem)]';
</script>

{#if entries.length > 0}
	<div class={containerClass}>
		{#if label !== ''}
			<div
				class={`mb-1 inline-block rounded px-1.5 py-0.5 text-[10px] font-bold uppercase tracking-wide shadow ${labelChipClass}`}
			>
				{label}
			</div>
		{/if}
		<div class={`grid ${overlayGridClass} gap-1`}>
			{#each entries as entry}
				<div
					class={`min-w-0 truncate rounded px-1.5 py-0.5 font-bold leading-tight shadow ${textSizeClass} ${baseChipClass}`}
				>
					{entry.name}{entry.count && entry.count > 1 ? ` ×${entry.count}` : ''}
				</div>
			{/each}
		</div>
	</div>
{/if}
