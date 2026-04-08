<script lang="ts">
	export let src = '';
	export let alt = '';
	export let className = '';

	let isLoaded = false;
	let previousSrc = '';

	$: if (src !== previousSrc) {
		previousSrc = src;
		isLoaded = false;
	}
</script>

<img
	{src}
	{alt}
	class={`${className} ${isLoaded ? 'opacity-100' : 'opacity-0'}`}
	style:visibility={isLoaded ? 'visible' : 'hidden'}
	on:load={() => (isLoaded = true)}
/>
