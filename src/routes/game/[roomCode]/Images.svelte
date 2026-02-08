<script lang="ts">
	import { http_host } from '$lib/gameServer';

	export let displayImages: string[];
	export let selectable = false;
	export let selectedImage = '';
	export let mode: 'hand' | 'board' = 'board';

	$: isHandMode = mode === 'hand';
	$: sectionClass = isHandMode
		? 'grid h-full w-full grid-cols-2 gap-3 lg:grid-cols-3 lg:grid-rows-2'
		: 'grid w-full grid-cols-2 gap-3 lg:grid-cols-3';
	$: imageClassBase = isHandMode
		? 'h-full w-full rounded-lg object-cover object-center aspect-[2/3]'
		: 'w-full rounded-lg object-cover object-center aspect-[2/3]';
</script>

<section class={sectionClass}>
	{#if selectable}
		{#each displayImages as image}
			<button type="button" class="group block h-full w-full" on:click={() => (selectedImage = image)}>
				<img
					class={`${selectedImage === image ? 'ring-4 ring-white shadow-xlg' : ''} ${imageClassBase} cursor-pointer transition-all duration-150 ease-in-out group-hover:scale-105 group-hover:shadow-2xl group-focus:shadow-2xl`}
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
