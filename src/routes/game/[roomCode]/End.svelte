<script lang="ts">
	import type { PlayerInfo } from '$lib/types';
	import { rankPlayersByPoints, type RankedPlayerEntry } from '$lib/ranking';

	export let players: { [key: string]: PlayerInfo } = {};
	let rankedPlayers: RankedPlayerEntry[] = [];

	$: {
		rankedPlayers = rankPlayersByPoints(players);
	}
</script>

<div class="flex w-80/10 justify-center">
	<div>
		<h1 class="text-4xl mb-10 text-center">Game Over!</h1>

		<div class="light p-4">
			<div>
				{#each rankedPlayers as entry}
					<div class="flex space-between w-44 text-xl">
						<div class="flex-auto">
							{entry.rank}.
							<span class={`${entry.isTopScore ? 'boujee-text' : ''} `}>{entry.name}</span>
						</div>
						<div class="font-right">
							{entry.points}
						</div>
					</div>
				{/each}
			</div>
		</div>
	</div>
</div>
