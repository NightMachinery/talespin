<script lang="ts">
	import tableSvg from '../../../../assets/SVG/table.svg?raw';
	import handSvg from '../../../../assets/SVG/hand.svg?raw';
	import resultsSvg from '../../../../assets/SVG/results.svg?raw';

	export let value: string;
	export let modes: { id: string; label: string; icon: 'table' | 'hand' | 'results' }[];

	const icons = {
		table: tableSvg,
		hand: handSvg,
		results: resultsSvg
	};
</script>

<div class="flex items-center rounded-lg bg-surface-200/50 p-1 dark:bg-surface-700/50">
	{#each modes as mode}
		<button
			type="button"
			class={`flex flex-1 items-center justify-center rounded-md p-2 transition-colors [&>svg]:h-5 [&>svg]:w-5 ${
				value === mode.id
					? 'bg-surface-50 text-surface-900 shadow-sm dark:bg-surface-800 dark:text-surface-50'
					: 'text-surface-600 hover:text-surface-900 dark:text-surface-400 dark:hover:text-surface-50'
			}`}
			title={mode.label}
			aria-label={mode.label}
			on:click={() => (value = mode.id)}
		>
			{@html icons[mode.icon]}
		</button>
	{/each}
</div>
