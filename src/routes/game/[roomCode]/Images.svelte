<script lang="ts">
	import { createEventDispatcher } from 'svelte';
	import { http_host } from '$lib/gameServer';
	import { CARD_IMAGE_ALT_TEXT } from '$lib/cardImageText';
	import { cardsFitToHeight } from '$lib/viewOptions';

	// Toggle dimming of cards that are not currently selected.
	const ENABLE_NON_SELECTED_CARD_DIM = false;
	const NON_SELECTED_CARD_DIM_CLASS = 'opacity-75';

	const dispatch = createEventDispatcher<{
		select: string;
	}>();

	export let displayImages: string[];
	export let selectable = false;
	export let selectedImages: string[] = [];
	export let mode: 'hand' | 'board' = 'board';
	export let disabledImages: string[] = [];

	$: isHandMode = mode === 'hand';
	$: handDesktopFitEnabled = isHandMode && $cardsFitToHeight;
	$: handDesktopRowCount = Math.max(2, Math.ceil(Math.max(displayImages?.length ?? 0, 1) / 3));
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
	$: selectedImageSet = new Set(
		selectedImages.filter(
			(image) => visibleImageSet.has(image) && !disabledImageSet.has(image)
		)
	);

	function selectImage(image: string, isDisabled: boolean) {
		if (!selectable || isDisabled) return;
		dispatch('select', image);
	}

	function cardContainerClass(image: string, isDisabled: boolean) {
		return `card-hover-source group relative block w-full overflow-visible rounded-lg focus-visible:outline-none ${handButtonSizeClass} ${
			selectedImageSet.size > 0 && !selectedImageSet.has(image) ? nonSelectedCardDimClass : ''
		} ${isDisabled ? 'cursor-not-allowed' : selectable ? '' : 'cursor-default'}`;
	}

	function cardImageClass(image: string, isDisabled: boolean) {
		const baseClass = `${imageClassBase} pointer-events-none transition-all duration-150 ease-out`;
		if (isDisabled) {
			return `${baseClass} cursor-not-allowed ring-[3px] ring-gray-400`;
		}
		if (selectedImageSet.has(image)) {
			return `${baseClass} brightness-105 ring-4 ring-white shadow-xlg`;
		}
		if (selectable) {
			return `${baseClass} card-hover-target cursor-pointer group-focus-visible:ring-2 group-focus-visible:ring-white/85 group-focus-visible:shadow-[0_0_0_2px_rgba(255,255,255,0.22),0_16px_30px_rgba(0,0,0,0.38)]`;
		}
		return baseClass;
	}
</script>

<section class={`${sectionClass} isolate`} style={handDesktopFitStyle}>
	{#each displayImages as image}
		{@const isDisabled = disabledImageSet.has(image)}
		{#if selectable}
			<button
				type="button"
				class={cardContainerClass(image, isDisabled)}
				disabled={isDisabled}
				on:click={() => selectImage(image, isDisabled)}
			>
				<img
					class={cardImageClass(image, isDisabled)}
					src={`${http_host}/cards/${image}`}
					alt={CARD_IMAGE_ALT_TEXT}
				/>
			</button>
		{:else}
			<div class={cardContainerClass(image, isDisabled)}>
				<img
					class={cardImageClass(image, isDisabled)}
					src={`${http_host}/cards/${image}`}
					alt={CARD_IMAGE_ALT_TEXT}
				/>
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
