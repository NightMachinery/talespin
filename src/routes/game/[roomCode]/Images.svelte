<script lang="ts">
	import { http_host } from '$lib/gameServer';

	// Toggle dimming of cards that are not currently selected.
	const ENABLE_NON_SELECTED_CARD_DIM = false;
	const NON_SELECTED_CARD_DIM_CLASS = 'opacity-75';

	export let displayImages: string[];
	export let selectable = false;
	export let selectedImage = '';
	export let mode: 'hand' | 'board' = 'board';
	export let disabledImages: string[] = [];

	$: isHandMode = mode === 'hand';
	$: sectionClass = isHandMode
		? 'grid h-full w-full grid-cols-2 gap-3 overflow-visible p-1 lg:grid-cols-3 lg:grid-rows-2'
		: 'grid w-full grid-cols-2 gap-3 overflow-visible p-1 lg:grid-cols-3';
	$: imageClassBase = isHandMode
		? 'h-full w-full rounded-lg object-cover object-center aspect-[2/3]'
		: 'w-full rounded-lg object-cover object-center aspect-[2/3]';
	$: nonSelectedCardDimClass = ENABLE_NON_SELECTED_CARD_DIM ? NON_SELECTED_CARD_DIM_CLASS : '';
	$: disabledImageSet = new Set(disabledImages);
	$: if (disabledImageSet.has(selectedImage)) {
		selectedImage = '';
	}
</script>

<section class={`${sectionClass} isolate`}>
	{#if selectable}
		{#each displayImages as image}
			{@const isDisabled = disabledImageSet.has(image)}
			<button
				type="button"
					  class={`group relative block h-full w-full overflow-visible rounded-lg focus-visible:outline-none ${
							selectedImage !== '' && selectedImage !== image ? nonSelectedCardDimClass : ''
							} ${isDisabled ? 'cursor-not-allowed' : ''}`}
					  disabled={isDisabled}
					  on:click={() => {
							   if (!isDisabled) {
								   selectedImage = image;
							   }
							   }}
				>
				<img
					class={`${imageClassBase} pointer-events-none transition-all duration-150 ease-out ${
						  isDisabled
						  ? 'cursor-not-allowed ring-[3px] ring-gray-400'
						  /* brightness-[65%] */
						  /* Greying the disabled card makes it difficult to view it, which might affect the gameplay adversely. */
						  : selectedImage === image
						  ? 'brightness-105 ring-4 ring-white shadow-xlg'
						  : 'cursor-pointer group-hover:ring-2 group-hover:ring-white/85 group-hover:brightness-105 group-hover:shadow-[0_0_0_2px_rgba(255,255,255,0.22),0_16px_30px_rgba(0,0,0,0.38)] group-focus-visible:ring-2 group-focus-visible:ring-white/85 group-focus-visible:shadow-[0_0_0_2px_rgba(255,255,255,0.22),0_16px_30px_rgba(0,0,0,0.38)]'
						  }`}
						  src={`${http_host}/cards/${image}`}
						  alt="You can't play this game without the images!"
				/>
			</button>
		{/each}
	{:else}
		{#each displayImages as image}
			<img
				class={`${imageClassBase} transition-all duration-150 ease-in-out`}
				src={`${http_host}/cards/${image}`}
				alt="You can't play this game without the images!"
			/>
		{/each}
	{/if}
</section>
