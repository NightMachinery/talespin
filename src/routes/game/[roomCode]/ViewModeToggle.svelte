<script lang="ts">
	export let value: string;
	export let modes: { id: string; label: string; icon: 'table' | 'hand' | 'results' }[];
</script>

<div class="flex items-center rounded-lg bg-surface-200/50 p-1 dark:bg-surface-700/50">
	{#each modes as mode}
		<button
			type="button"
			class={`flex flex-1 items-center justify-center rounded-md p-2 transition-colors ${
				value === mode.id
					? 'bg-surface-50 text-surface-900 shadow-sm dark:bg-surface-800 dark:text-surface-50'
					: 'text-surface-600 hover:text-surface-900 dark:text-surface-400 dark:hover:text-surface-50'
			}`}
			title={mode.label}
			aria-label={mode.label}
			on:click={() => (value = mode.id)}
		>
			{#if mode.icon === 'table'}
				<svg
					xmlns="http://www.w3.org/2000/svg"
					viewBox="0 0 24 24"
					fill="none"
					stroke="currentColor"
					stroke-width="2"
					stroke-linecap="round"
					stroke-linejoin="round"
					class="h-5 w-5"
				>
					<rect x="3" y="3" width="7" height="7"></rect>
					<rect x="14" y="3" width="7" height="7"></rect>
					<rect x="14" y="14" width="7" height="7"></rect>
					<rect x="3" y="14" width="7" height="7"></rect>
				</svg>
			{:else if mode.icon === 'hand'}
				<svg
					xmlns="http://www.w3.org/2000/svg"
					viewBox="0 0 24 24"
					fill="none"
					stroke="currentColor"
					stroke-width="2"
					stroke-linecap="round"
					stroke-linejoin="round"
					class="h-5 w-5"
				>
					<rect x="2" y="5" width="20" height="14" rx="2"></rect>
					<path d="M8 5v14"></path>
					<path d="M16 5v14"></path>
				</svg>
			{:else if mode.icon === 'results'}
				<svg
					xmlns="http://www.w3.org/2000/svg"
					viewBox="0 0 24 24"
					fill="none"
					stroke="currentColor"
					stroke-width="2"
					stroke-linecap="round"
					stroke-linejoin="round"
					class="h-5 w-5"
				>
					<path d="M3 3v18h18"></path>
					<path d="M18 17V9"></path>
					<path d="M13 17V5"></path>
					<path d="M8 17v-3"></path>
				</svg>
			{/if}
		</button>
	{/each}
</div>
