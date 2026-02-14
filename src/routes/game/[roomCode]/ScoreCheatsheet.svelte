<script lang="ts">
	import type { PlayerInfo } from '$lib/types';

	export let players: { [key: string]: PlayerInfo } = {};
	export let activePlayer = '';
	export let storytellerLossComplement = 0;
	export let votesPerGuesser = 1;
	export let votesPerGuesserMax = 1;

	$: guesserCount = Math.max(0, Object.keys(players).length - 1);
	$: effectiveLossThreshold = Math.max(0, guesserCount - storytellerLossComplement);
	$: effectiveVotesPerGuesser = Math.max(
		1,
		Math.min(votesPerGuesser, Math.max(votesPerGuesserMax, 1))
	);
	$: storytellerLabel = activePlayer || 'Storyteller';
</script>

<div class="card light p-4">
	<h2 class="mb-2 text-lg font-semibold">How points work</h2>
	<p class="mb-2 text-xs opacity-80">
		There are <span class="font-semibold">{guesserCount}</span> guessers. With C=<span
			class="font-semibold">{storytellerLossComplement}</span
		>, storyteller-loss threshold is
		<span class="font-semibold">{effectiveLossThreshold}</span>.
	</p>
	<ul class="ml-5 list-disc space-y-1 text-sm">
		<li>
			Loss scenario triggers when right guesses ≥ {effectiveLossThreshold} or wrong guesses ≥
			{effectiveLossThreshold}.
		</li>
		<li>
			Loss scenario points: <span class="font-semibold">{storytellerLabel}</span> +0, each guesser +2.
		</li>
		<li>
			Otherwise points: <span class="font-semibold">{storytellerLabel}</span> +3, each guesser with at
			least one correct vote +3.
		</li>
		<li>
			Double-correct bonus: +1 if 2+ of your {effectiveVotesPerGuesser} vote
			{effectiveVotesPerGuesser === 1 ? '' : 's'} hit storyteller card.
		</li>
		<li>Decoy bonus: +1 per vote token on your card (max +3, non-storyteller only).</li>
	</ul>
</div>
