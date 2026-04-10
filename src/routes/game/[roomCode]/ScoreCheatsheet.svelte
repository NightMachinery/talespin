<script lang="ts">
	import type { BeautyResultsDisplayMode, GameMode } from '$lib/types';

	export let gameMode: GameMode = 'dixit_plus';
	export let activePlayer = '';
	export let votesPerGuesser = 1;
	export let votesPerGuesserMax = 1;
	export let bonusCorrectGuessOnThresholdCorrectLoss = true;
	export let bonusDoubleVoteOnThresholdCorrectLoss = true;
	export let bonusThresholdLossTogglesApplyToAllStorytellerLossRounds = true;
	export let beautyEnabled = false;
	export let beautyVotesPerPlayer = 1;
	export let beautyVotesPerPlayerMax = 1;
	export let beautyAllowDuplicateVotes = false;
	export let beautyPointsBonus = 2;
	export let beautyResultsDisplayMode: BeautyResultsDisplayMode = 'combined';

	$: effectiveVotesPerGuesser = Math.max(
		1,
		Math.min(votesPerGuesser, Math.max(votesPerGuesserMax, 1))
	);
	$: storytellerLabel = activePlayer || 'Storyteller';
	$: effectiveBeautyVotesPerPlayer = Math.max(
		1,
		Math.min(beautyVotesPerPlayer, Math.max(beautyVotesPerPlayerMax, 1))
	);
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
				Fallen: if nobody else matched, the scout "falls", gets 0 and cannot score more reveals this
				round.
			</li>
			<li>
				Dark player: the player with most associated cards is "in the dark." If they made any
				mistakes, each scored Spark/Super-Spark loses 1 base star.
			</li>
		</ul>
	{:else}
		<p class="mb-2 text-xs opacity-80">
			Storyteller-loss round = storyteller loses because enough guessers were correct or wrong.
		</p>
		<ul class="ml-5 list-disc space-y-1 text-sm">
			<li>
				Storyteller-loss round: <span class="font-semibold">{storytellerLabel}</span> +0, each guesser
				+2.
			</li>
			{#if bonusThresholdLossTogglesApplyToAllStorytellerLossRounds}
				<li>The two bonus toggles below apply in any storyteller-loss round.</li>
			{:else}
				<li>The two bonus toggles below apply only when enough guessers were correct.</li>
			{/if}
			{#if bonusCorrectGuessOnThresholdCorrectLoss}
				<li>
					Correct-guess bonus: in covered storyteller-loss rounds, guessers with at least one
					correct vote get +3 base (instead of +2).
				</li>
			{:else}
				<li>Correct-guess bonus: off in covered storyteller-loss rounds.</li>
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
					{effectiveVotesPerGuesser === 1 ? '' : 's'} hit storyteller card, except in covered storyteller-loss
					rounds.
				</li>
			{/if}
			<li>Decoy bonus: +1 per vote token on your card (max +3, non-storyteller only).</li>
			{#if beautyEnabled}
				<li>
					Most Beautiful: every active player casts {effectiveBeautyVotesPerPlayer} beauty vote
					{effectiveBeautyVotesPerPlayer === 1 ? '' : 's'} on other players’ cards.
				</li>
				<li>
					Beauty winners: each top-voted owner gets +{beautyPointsBonus}
					{beautyPointsBonus === 1 ? ' point' : ' points'} once per round.
				</li>
				<li>
					Beauty duplicates: {beautyAllowDuplicateVotes ? 'allowed' : 'not allowed'}. Reveal mode: {beautyResultsDisplayMode}.
				</li>
			{/if}
		</ul>
	{/if}
</div>
