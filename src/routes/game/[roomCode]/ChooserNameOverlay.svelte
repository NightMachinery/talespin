<script lang="ts">
	import { transparentCardNameOverlays } from '$lib/viewOptions';
	import type { BeautyBadgeTier } from '$lib/beautyResults';

	type ChooserOverlayEntry = {
		name: string;
		count?: number;
	};

	export let entries: ChooserOverlayEntry[] = [];
	export let avoidTopLeftBadge = false;
	export let label = '';
	export let labelTier: BeautyBadgeTier = 'default';
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
				: 'border border-transparent bg-fuchsia-700/85 text-fuchsia-50'
			: $transparentCardNameOverlays
				? 'border border-sky-100/70 bg-slate-900/15 text-sky-50'
				: 'border border-transparent bg-slate-900/80 text-sky-50';
	$: labelChipClass =
		tone === 'beauty' && labelTier === 'gold'
			? $transparentCardNameOverlays
				? 'border border-amber-100/80 bg-gradient-to-r from-amber-300/35 via-yellow-200/25 to-amber-400/35 text-amber-50'
				: 'border border-amber-200/90 bg-gradient-to-r from-amber-200 via-yellow-300 to-amber-400 text-amber-950'
			: tone === 'beauty' && labelTier === 'silver'
				? $transparentCardNameOverlays
					? 'border border-slate-100/80 bg-gradient-to-r from-slate-200/30 via-zinc-100/20 to-slate-300/30 text-slate-50'
					: 'border border-slate-200/90 bg-gradient-to-r from-slate-100 via-slate-300 to-zinc-300 text-slate-950'
				: tone === 'beauty' && labelTier === 'bronze'
					? $transparentCardNameOverlays
						? 'border border-orange-100/80 bg-gradient-to-r from-orange-300/35 via-amber-700/20 to-orange-400/35 text-orange-50'
						: 'border border-orange-200/90 bg-gradient-to-r from-orange-200 via-amber-400 to-orange-500 text-orange-950'
					: tone === 'beauty'
						? $transparentCardNameOverlays
							? 'border border-rose-200/75 bg-rose-500/20 text-rose-50'
							: 'border border-transparent bg-rose-500/90 text-black'
						: $transparentCardNameOverlays
							? 'border border-sky-100/75 bg-sky-500/20 text-sky-50'
							: 'border border-transparent bg-sky-400/90 text-black';
	$: containerClass =
		position === 'bottom-right'
			? 'pointer-events-none absolute bottom-10 right-2 z-20 max-w-[calc(100%-1rem)]'
			: position === 'bottom-left'
				? 'pointer-events-none absolute bottom-10 left-2 z-20 max-w-[calc(100%-1rem)]'
				: avoidTopLeftBadge
					? 'pointer-events-none absolute left-2 top-10 z-20 max-w-[calc(100%-1rem)]'
					: 'pointer-events-none absolute right-2 top-2 z-20 max-w-[calc(100%-1rem)]';
</script>

{#if entries.length > 0 || label !== ''}
	<div class={containerClass}>
		{#if label !== ''}
			<div
				class={`mb-1 inline-flex max-w-full items-center rounded px-1.5 py-0.5 text-[11px] font-bold leading-tight shadow ${labelChipClass}`}
			>
				{label}
			</div>
		{/if}
		{#if entries.length > 0}
			<div class={`grid ${overlayGridClass} gap-1`}>
				{#each entries as entry}
					<div
						class={`min-w-0 truncate rounded px-1.5 py-0.5 font-bold leading-tight shadow ${textSizeClass} ${baseChipClass}`}
					>
						{entry.name}{entry.count && entry.count > 1 ? ` ×${entry.count}` : ''}
					</div>
				{/each}
			</div>
		{/if}
	</div>
{/if}
