<script lang="ts">
	export let actions: { label: string; disabled?: boolean; onClick: () => void }[] = [];
</script>

<div class="space-y-2">
	{#each actions as action (action.label)}
		<button
			class="btn variant-filled w-full"
			disabled={action.disabled ?? false}
			on:click={action.onClick}
		>
			{action.label}
		</button>
	{/each}
</div>
