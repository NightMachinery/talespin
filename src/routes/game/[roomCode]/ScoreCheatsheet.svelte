<script lang="ts">
	import type { GameMode } from '$lib/types';

	export let gameMode: GameMode = 'dixit_plus';
	export let activePlayer = '';
	export let votesPerGuesser = 1;
	export let votesPerGuesserMax = 1;
	export let bonusCorrectGuessOnThresholdCorrectLoss = true;
	export let bonusDoubleVoteOnThresholdCorrectLoss = true;

	$: effectiveVotesPerGuesser = Math.max(
		1,
		Math.min(votesPerGuesser, Math.max(votesPerGuesserMax, 1))
	);
	$: storytellerLabel = activePlayer || 'Storyteller';
</script>

<div class="card light p-4">
	<h2 class="mb-2 text-lg font-semibold">How points work</h2>
	{#if gameMode === 'stella'}
		<ul class="ml-5 list-disc space-y-1 text-sm">
			<li>
				Spark: if 2+ other players matched the scout card, each matching non-fallen player gets 2.
			</li>
			<li>
				Super-Spark: if exactly 1 other player matched, both matching non-fallen players get 3.
			</li>
			<li>
				Fall: if nobody else matched, the scout gets 0 and cannot score more reveals this round.
			</li>
			<li>
				Dark player: if they made any mistakes, each scored Spark/Super-Spark loses 1 base star.
			</li>
		</ul>
	{:else}
		<p class="mb-2 text-xs opacity-80">
			Threshold-correct storyteller-loss round = storyteller loses because enough guessers were
			correct.
		</p>
		<ul class="ml-5 list-disc space-y-1 text-sm">
			<li>
				Storyteller-loss round: <span class="font-semibold">{storytellerLabel}</span> +0, each guesser
				+2.
			</li>
			{#if bonusCorrectGuessOnThresholdCorrectLoss}
				<li>
					In threshold-correct storyteller-loss rounds, guessers with at least one correct vote get
					+3 base (instead of +2).
				</li>
			{:else}
				<li>
					In threshold-correct storyteller-loss rounds, correct guessers stay at +2 base (bonus is
					off).
				</li>
			{/if}
			<li>
				Normal round: <span class="font-semibold">{storytellerLabel}</span> +3, each guesser with at
				least one correct vote +3.
			</li>
			{#if bonusDoubleVoteOnThresholdCorrectLoss}
				<li>
					Double-correct bonus: +1 if 2+ of your {effectiveVotesPerGuesser} vote
					{effectiveVotesPerGuesser === 1 ? '' : 's'} hit storyteller card.
				</li>
			{:else}
				<li>
					Double-correct bonus: +1 if 2+ of your {effectiveVotesPerGuesser} vote
					{effectiveVotesPerGuesser === 1 ? '' : 's'} hit storyteller card, except in threshold-correct
					storyteller-loss rounds.
				</li>
			{/if}
			<li>Decoy bonus: +1 per vote token on your card (max +3, non-storyteller only).</li>
		</ul>
	{/if}
</div>
