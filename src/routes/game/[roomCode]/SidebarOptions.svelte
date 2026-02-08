<script lang="ts">
	import { cardsFitToHeight } from '$lib/viewOptions';
</script>

<div class="card light space-y-3 p-4">
	<h2 class="text-lg font-semibold">Options</h2>
	<label class="flex items-start gap-3">
		<input
			type="checkbox"
			class="mt-1 h-4 w-4 cursor-pointer accent-primary-500"
			bind:checked={$cardsFitToHeight}
		/>
		<span class="block font-medium">Cards fit to height</span>
	</label>
</div>
