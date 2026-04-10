<script lang="ts">
	import {
		mostBeautifulStats,
		mostBeautifulStatsError,
		mostBeautifulStatsLoading,
		type MostBeautifulPlayerStats
	} from '$lib/mostBeautiful';

	export let title = 'Most Beautiful ranking';
	export let compact = false;

	function rankPlayers(entries: MostBeautifulPlayerStats[]) {
		const sorted = [...entries]
			.filter((entry) => entry.votes_received > 0 || entry.rounds_won > 0)
			.sort(
				(a, b) =>
					b.rounds_won - a.rounds_won ||
					b.votes_received - a.votes_received ||
					a.display_name.localeCompare(b.display_name)
			);

		let previousEntry: MostBeautifulPlayerStats | null = null;
		let previousRank = 0;

		return sorted.map((entry, index) => {
			const rank =
				previousEntry !== null &&
				previousEntry.rounds_won === entry.rounds_won &&
				previousEntry.votes_received === entry.votes_received
					? previousRank
					: index + 1;
			previousEntry = entry;
			previousRank = rank;
			return { ...entry, rank };
		});
	}

	$: rankedStats = rankPlayers($mostBeautifulStats);
</script>

<div class={`card light ${compact ? 'p-3' : 'p-4'}`}>
	<h2 class={`${compact ? 'text-lg' : 'text-xl'} font-semibold`}>{title}</h2>
	{#if $mostBeautifulStatsLoading && rankedStats.length === 0}
		<p class="mt-2 text-sm opacity-70">Loading…</p>
	{:else if $mostBeautifulStatsError}
		<p class="mt-2 text-sm text-error-500">{$mostBeautifulStatsError}</p>
	{:else if rankedStats.length === 0}
		<p class="mt-2 text-sm opacity-70">No Most Beautiful stats yet.</p>
	{:else}
		<div class="mt-3 space-y-3">
			{#each rankedStats as entry}
				<div>
					<div class="font-semibold">
						{entry.rank}. {entry.display_name}: got {entry.votes_received} vote{entry.votes_received === 1
							? ''
							: 's'}, won {entry.rounds_won} round{entry.rounds_won === 1 ? '' : 's'} ({entry.tie_round_wins}
						tie{entry.tie_round_wins === 1 ? '' : 's'}, {entry.decisive_round_wins} decisive)
					</div>
					{#if entry.voters.filter((voter) => voter.votes > 0).length > 0}
						<div class="mt-1 pl-4 text-sm opacity-85">
							{#each entry.voters.filter((voter) => voter.votes > 0) as voter}
								<div>{voter.display_name}: {voter.votes}</div>
							{/each}
						</div>
					{/if}
				</div>
			{/each}
		</div>
	{/if}
</div>
