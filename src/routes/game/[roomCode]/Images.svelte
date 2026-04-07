<script lang="ts">
	import { createEventDispatcher } from 'svelte';
	import { getDesktopFitRowCount } from '$lib/cardGrid';
	import { http_host } from '$lib/gameServer';
	import { CARD_IMAGE_ALT_TEXT } from '$lib/cardImageText';
	import { cardsFitToHeight } from '$lib/viewOptions';

	// Toggle dimming of cards that are not currently selected.
	const ENABLE_NON_SELECTED_CARD_DIM = false;
	const NON_SELECTED_CARD_DIM_CLASS = 'opacity-75';

	const dispatch = createEventDispatcher<{
		select: string;
	}>();

	type ImageAnnotation = {
		label: string;
		className?: string;
	};

	export let displayImages: string[];
	export let selectable = false;
	export let selectableImages: string[] = [];
	export let selectedImages: string[] = [];
	export let mode: 'hand' | 'board' = 'board';
	export let disabledImages: string[] = [];
	export let imageAnnotations: Record<string, ImageAnnotation> = {};
	export let showIndexOverlay = false;

	$: isHandMode = mode === 'hand';
	$: handDesktopFitEnabled = isHandMode && $cardsFitToHeight;
	$: handDesktopRowCount = getDesktopFitRowCount(displayImages?.length);
	$: handDesktopGridClass = handDesktopFitEnabled
		? 'hand-fit-grid lg:h-full lg:grid-cols-3 lg:content-stretch'
		: 'lg:grid-cols-3 lg:auto-rows-max lg:content-start';
	$: handButtonSizeClass = handDesktopFitEnabled ? 'lg:h-full' : '';
	$: handDesktopFitStyle = handDesktopFitEnabled
		? `--hand-desktop-rows: ${handDesktopRowCount};`
		: '';
	$: sectionClass = isHandMode
		? `grid w-full grid-cols-2 gap-3 overflow-visible p-1 ${handDesktopGridClass}`
		: 'grid w-full grid-cols-2 gap-3 overflow-visible p-1 lg:grid-cols-3 lg:auto-rows-max lg:content-start';
	$: imageClassBase = isHandMode
		? `w-full rounded-lg object-cover object-center aspect-[2/3] ${
				handDesktopFitEnabled ? 'lg:h-full' : ''
			}`
		: 'w-full rounded-lg object-cover object-center aspect-[2/3]';
	$: nonSelectedCardDimClass = ENABLE_NON_SELECTED_CARD_DIM ? NON_SELECTED_CARD_DIM_CLASS : '';
	$: visibleImageSet = new Set(displayImages);
	$: disabledImageSet = new Set(disabledImages);
	$: hasSelectableImageRestriction = selectableImages.length > 0;
	$: selectableImageSet = new Set(
		selectableImages.filter((image) => visibleImageSet.has(image) && !disabledImageSet.has(image))
	);
	$: selectedImageSet = new Set(
		selectedImages.filter((image) => visibleImageSet.has(image) && !disabledImageSet.has(image))
	);

	function selectImage(image: string, isDisabled: boolean, canSelect: boolean) {
		if (!selectable || isDisabled || !canSelect) return;
		dispatch('select', image);
	}
</script>

<section class={`${sectionClass} isolate`} style={handDesktopFitStyle}>
	{#each displayImages as image, imageIndex}
		{@const isDisabled = disabledImageSet.has(image)}
		{@const isSelected = selectedImageSet.has(image)}
		{@const annotation = imageAnnotations[image]}
		{@const shouldDim = selectedImageSet.size > 0 && !isSelected}
		{@const canSelect =
			selectable &&
			!isDisabled &&
			(!hasSelectableImageRestriction || selectableImageSet.has(image))}
		{@const containerClass = `card-hover-source group relative block w-full overflow-visible rounded-lg focus-visible:outline-none ${handButtonSizeClass} ${
			shouldDim ? nonSelectedCardDimClass : ''
		} ${isDisabled ? 'cursor-not-allowed' : canSelect ? '' : 'cursor-default'}`}
		{@const imageClass = `${imageClassBase} pointer-events-none transition-all duration-150 ease-out ${
			isDisabled
				? 'cursor-not-allowed ring-[3px] ring-gray-400'
				: isSelected
					? 'brightness-105 ring-4 ring-white shadow-xlg'
					: canSelect
						? 'card-hover-target cursor-pointer group-focus-visible:ring-2 group-focus-visible:ring-white/85 group-focus-visible:shadow-[0_0_0_2px_rgba(255,255,255,0.22),0_16px_30px_rgba(0,0,0,0.38)]'
						: ''
		}`}
		{#if selectable}
			<button
				type="button"
				class={containerClass}
				disabled={isDisabled}
				aria-disabled={!canSelect}
				tabindex={canSelect ? 0 : -1}
				on:click={() => selectImage(image, isDisabled, canSelect)}
			>
				<img class={imageClass} src={`${http_host}/cards/${image}`} alt={CARD_IMAGE_ALT_TEXT} />
				{#if showIndexOverlay}
					<div
						class="pointer-events-none absolute right-2 top-2 z-20 rounded bg-black/70 px-2 py-0.5 text-xs font-bold text-white shadow"
					>
						#{imageIndex + 1}
					</div>
				{/if}
				{#if annotation}
					<div
						class={`pointer-events-none absolute inset-x-2 bottom-2 z-20 rounded px-2 py-1 text-center text-sm font-bold shadow ${annotation.className ?? 'bg-black/75 text-white'}`}
					>
						{annotation.label}
					</div>
				{/if}
			</button>
		{:else}
			<div class={containerClass}>
				<img class={imageClass} src={`${http_host}/cards/${image}`} alt={CARD_IMAGE_ALT_TEXT} />
				{#if showIndexOverlay}
					<div
						class="pointer-events-none absolute right-2 top-2 z-20 rounded bg-black/70 px-2 py-0.5 text-xs font-bold text-white shadow"
					>
						#{imageIndex + 1}
					</div>
				{/if}
				{#if annotation}
					<div
						class={`pointer-events-none absolute inset-x-2 bottom-2 z-20 rounded px-2 py-1 text-center text-sm font-bold shadow ${annotation.className ?? 'bg-black/75 text-white'}`}
					>
						{annotation.label}
					</div>
				{/if}
			</div>
		{/if}
	{/each}
</section>

<style>
	@media (min-width: 1024px) {
		.hand-fit-grid {
			grid-template-rows: repeat(var(--hand-desktop-rows, 2), minmax(0, 1fr));
		}
	}
</style>
