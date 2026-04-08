<script lang="ts">
	import type { ObserverInfo, PlayerInfo } from '$lib/types';
	import { rankPlayersByPoints, type RankedPlayerEntry } from '$lib/ranking';

	export let players: { [key: string]: PlayerInfo } = {};
	export let observers: { [key: string]: ObserverInfo } = {};
	let rankedPlayers: RankedPlayerEntry[] = [];
	let sortedObserverEntries: [string, ObserverInfo][] = [];

	$: {
		rankedPlayers = rankPlayersByPoints(players);
	}
	$: sortedObserverEntries = Object.entries(observers).sort(([a], [b]) => a.localeCompare(b));
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
			{#if sortedObserverEntries.length > 0}
				<div class="mt-4 border-t border-white/15 pt-3">
					<h2 class="mb-2 text-lg font-semibold">Observers</h2>
					{#each sortedObserverEntries as [observerName, observerInfo]}
						<div class="flex space-between w-44 text-xl">
							<div class="flex-auto">{observerName}</div>
							<div class="font-right">
								{observerInfo.points === null ? 'NA' : observerInfo.points}
							</div>
						</div>
					{/each}
				</div>
			{/if}
		</div>
	</div>
</div>
