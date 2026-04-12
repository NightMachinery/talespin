<script lang="ts">
	import {
		mostBeautifulStatsGamesLimit,
		mostBeautifulStats,
		mostBeautifulStatsError,
		mostBeautifulStatsLoading,
		refreshMostBeautifulStats,
		setMostBeautifulStatsGamesLimit,
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

	async function handleGamesLimitChange(event: Event) {
		const input = event.currentTarget as HTMLInputElement;
		const parsed = Number(input.value);
		if (!Number.isInteger(parsed) || parsed < 0) {
			input.value = `${$mostBeautifulStatsGamesLimit}`;
			return;
		}
		if (parsed !== $mostBeautifulStatsGamesLimit) {
			setMostBeautifulStatsGamesLimit(parsed);
			await refreshMostBeautifulStats();
		}
	}
</script>

<div class={`card light ${compact ? 'p-3' : 'p-4'}`}>
	<div class="flex items-end justify-between gap-3">
		<h2 class={`${compact ? 'text-lg' : 'text-xl'} font-semibold`}>{title}</h2>
		<label class="text-right text-xs opacity-80">
			<span class="mb-1 block font-semibold uppercase tracking-wide opacity-70">Last N games</span>
			<input
				type="number"
				class="w-20 rounded border px-2 py-1 text-right text-gray-700 shadow"
				min="0"
				step="1"
				value={$mostBeautifulStatsGamesLimit}
				on:change={handleGamesLimitChange}
			/>
		</label>
	</div>
	<p class="mt-2 text-xs opacity-70">0 = all history. Showing up to the top 30 players.</p>
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
						{entry.rank}. {entry.display_name} — {entry.rounds_won}W / {entry.tie_round_wins}T / {entry.decisive_round_wins}D
						· {entry.votes_received}
						vote{entry.votes_received === 1 ? '' : 's'}
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
