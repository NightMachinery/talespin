<script lang="ts">
	import { createEventDispatcher } from 'svelte';
	import { getDesktopFitRowCount } from '$lib/cardGrid';
	import { http_host } from '$lib/gameServer';
	import { CARD_IMAGE_ALT_TEXT } from '$lib/cardImageText';
	import CardImage from '$lib/CardImage.svelte';
	import { cardsFitToHeight } from '$lib/viewOptions';
	import ChooserNameOverlay from './ChooserNameOverlay.svelte';

	// Toggle dimming of cards that are not currently selected.
	const ENABLE_NON_SELECTED_CARD_DIM = false;
	const NON_SELECTED_CARD_DIM_CLASS = 'opacity-75';

	const dispatch = createEventDispatcher<{
		select: string;
		pinToggle: string;
	}>();

	type ImageAnnotation = {
		label: string;
		className?: string;
	};

	type ChooserOverlayEntry = {
		name: string;
		count?: number;
	};

	export let displayImages: string[];
	export let selectable = false;
	export let selectableImages: string[] = [];
	export let selectedImages: string[] = [];
	export let mode: 'hand' | 'board' = 'board';
	export let disabledImages: string[] = [];
	export let imageAnnotations: Record<string, ImageAnnotation> = {};
	export let imageChooserOverlays: Record<string, ChooserOverlayEntry[]> = {};
	export let imageHighlightClasses: Record<string, string> = {};
	export let showIndexOverlay = false;
	export let indexOverlayPosition: 'left' | 'right' = 'right';
	export let indexOverlayLabels: Array<string | number> = [];
	export let imageSecondaryBadges: Record<string, string | number> = {};
	export let secondaryBadgePosition: 'left' | 'right' = 'left';
	export let desktopFitToHeight = false;
	export let pinnedImages: string[] = [];
	export let showPinBadges = false;
	export let pinTogglesEnabled = false;

	$: isHandMode = mode === 'hand';
	$: desktopFitEnabled = $cardsFitToHeight && (isHandMode || desktopFitToHeight);
	$: desktopFitRowCount = getDesktopFitRowCount(displayImages?.length);
	$: desktopGridClass = desktopFitEnabled
		? 'desktop-fit-grid lg:h-full lg:grid-cols-3 lg:content-stretch'
		: 'lg:grid-cols-3 lg:auto-rows-max lg:content-start';
	$: desktopButtonSizeClass = desktopFitEnabled ? 'lg:h-full' : '';
	$: desktopFitStyle = desktopFitEnabled ? `--desktop-fit-rows: ${desktopFitRowCount};` : '';
	$: sectionClass = `grid w-full grid-cols-2 gap-3 overflow-visible p-1 ${desktopGridClass}`;
	$: imageClassBase = `w-full rounded-lg object-cover object-center aspect-[2/3] ${
		desktopFitEnabled ? 'lg:h-full' : ''
	}`;
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
	$: indexOverlayPositionClass = indexOverlayPosition === 'left' ? 'left-2' : 'right-2';
	$: secondaryBadgePositionClass = secondaryBadgePosition === 'left' ? 'left-2' : 'right-2';
	$: pinnedImageSet = new Set(pinnedImages.filter((image) => visibleImageSet.has(image)));

	function selectImage(image: string, isDisabled: boolean, canSelect: boolean) {
		if (!selectable || isDisabled || !canSelect) return;
		dispatch('select', image);
	}

	function togglePin(event: MouseEvent, image: string) {
		event.preventDefault();
		event.stopPropagation();
		if (!pinTogglesEnabled) return;
		dispatch('pinToggle', image);
	}

	function handlePinKeydown(event: KeyboardEvent, image: string) {
		if (event.key !== 'Enter' && event.key !== ' ') return;
		event.preventDefault();
		event.stopPropagation();
		if (!pinTogglesEnabled) return;
		dispatch('pinToggle', image);
	}
</script>

<section class={`${sectionClass} isolate`} style={desktopFitStyle}>
	{#each displayImages as image, imageIndex}
		{@const isDisabled = disabledImageSet.has(image)}
		{@const isSelected = selectedImageSet.has(image)}
		{@const annotation = imageAnnotations[image]}
		{@const chooserEntries = imageChooserOverlays[image] ?? []}
		{@const highlightClass = imageHighlightClasses[image] ?? ''}
		{@const indexOverlayLabel = indexOverlayLabels[imageIndex] ?? imageIndex + 1}
		{@const secondaryBadgeLabel = imageSecondaryBadges[image]}
		{@const isPinned = pinnedImageSet.has(image)}
		{@const shouldDim = selectedImageSet.size > 0 && !isSelected}
		{@const canSelect =
			selectable &&
			!isDisabled &&
			(!hasSelectableImageRestriction || selectableImageSet.has(image))}
		{@const containerClass = `card-hover-source group relative block w-full overflow-visible rounded-lg bg-slate-900/35 focus-visible:outline-none ${desktopButtonSizeClass} ${
			shouldDim ? nonSelectedCardDimClass : ''
		} ${isDisabled ? 'cursor-not-allowed' : canSelect ? '' : 'cursor-default'}`}
		{@const imageClass = `${imageClassBase} pointer-events-none transition-all duration-150 ease-out ${
			isDisabled
				? 'cursor-not-allowed ring-[3px] ring-gray-400'
				: highlightClass
					? highlightClass
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
				<CardImage
					className={imageClass}
					src={`${http_host}/cards/${image}`}
					alt={CARD_IMAGE_ALT_TEXT}
				/>
				{#if showIndexOverlay}
					<div
						class={`pointer-events-none absolute ${indexOverlayPositionClass} top-2 z-20 rounded bg-black/70 px-2 py-0.5 text-xs font-bold text-white shadow`}
					>
						#{indexOverlayLabel}
					</div>
				{/if}
				{#if secondaryBadgeLabel !== undefined}
					<div
						class={`pointer-events-none absolute ${secondaryBadgePositionClass} top-10 z-20 rounded bg-primary-500/90 px-2 py-0.5 text-xs font-bold text-black shadow`}
					>
						Q{secondaryBadgeLabel}
					</div>
				{/if}
				{#if showPinBadges}
					<!-- svelte-ignore a11y-no-noninteractive-tabindex -->
					<span
						role={pinTogglesEnabled ? 'button' : 'img'}
						tabindex={-1}
						aria-label={isPinned ? 'Unpin card' : 'Pin card'}
						title={isPinned ? 'Pinned card' : 'Pin this card'}
						class={`absolute left-2 top-2 z-30 rounded-full px-2 py-1 text-xs font-bold shadow ${
							isPinned
								? 'bg-warning-400 text-black'
								: pinTogglesEnabled
									? 'bg-black/60 text-white opacity-80'
									: 'bg-black/45 text-white/70'
						}`}
						on:click={(event) => togglePin(event, image)}
						on:keydown={(event) => handlePinKeydown(event, image)}
					>
						📌
					</span>
				{/if}
				<ChooserNameOverlay entries={chooserEntries} />
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
				<CardImage
					className={imageClass}
					src={`${http_host}/cards/${image}`}
					alt={CARD_IMAGE_ALT_TEXT}
				/>
				{#if showIndexOverlay}
					<div
						class={`pointer-events-none absolute ${indexOverlayPositionClass} top-2 z-20 rounded bg-black/70 px-2 py-0.5 text-xs font-bold text-white shadow`}
					>
						#{indexOverlayLabel}
					</div>
				{/if}
				{#if secondaryBadgeLabel !== undefined}
					<div
						class={`pointer-events-none absolute ${secondaryBadgePositionClass} top-10 z-20 rounded bg-primary-500/90 px-2 py-0.5 text-xs font-bold text-black shadow`}
					>
						Q{secondaryBadgeLabel}
					</div>
				{/if}
				{#if showPinBadges}
					<!-- svelte-ignore a11y-no-noninteractive-tabindex -->
					<span
						role={pinTogglesEnabled ? 'button' : 'img'}
						tabindex={-1}
						aria-label={isPinned ? 'Unpin card' : 'Pin card'}
						title={isPinned ? 'Pinned card' : 'Pin this card'}
						class={`absolute left-2 top-2 z-30 rounded-full px-2 py-1 text-xs font-bold shadow ${
							isPinned
								? 'bg-warning-400 text-black'
								: pinTogglesEnabled
									? 'bg-black/60 text-white opacity-80'
									: 'bg-black/45 text-white/70'
						}`}
						on:click={(event) => togglePin(event, image)}
						on:keydown={(event) => handlePinKeydown(event, image)}
					>
						📌
					</span>
				{/if}
				<ChooserNameOverlay entries={chooserEntries} />
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
		.desktop-fit-grid {
			grid-template-rows: repeat(var(--desktop-fit-rows, 2), minmax(0, 1fr));
		}
	}
</style>
