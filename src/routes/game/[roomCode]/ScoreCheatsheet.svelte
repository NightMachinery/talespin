<script lang="ts">
	import {
		beautyScoringMode,
		beautyVotePointsDivisorEffective,
		beautyVotePointsDivisorMode
	} from '$lib/mostBeautiful';
	import { clueRatingEnabled, clueRatingMaxStars } from '$lib/clueRating';
	import type { BeautyResultsDisplayMode, GameMode } from '$lib/types';

	export let gameMode: GameMode = 'dixit_plus';
	export let activePlayer = '';
	export let storytellerLossComplement = 0;
	export let storytellerSuccessPoints = 3;
	export let votesPerGuesser = 1;
	export let votesPerGuesserMax = 1;
	export let bonusCorrectGuessOnThresholdCorrectLoss = true;
	export let doubleVoteBonusNormalPoints = 1;
	export let doubleVoteBonusTooManyWrongPoints = 1;
	export let doubleVoteBonusTooManyWrongFollowsNormal = true;
	export let doubleVoteBonusTooManyCorrectPoints = 1;
	export let doubleVoteBonusTooManyCorrectFollowsNormal = true;
	export let bonusThresholdLossTogglesApplyToAllStorytellerLossRounds = true;
	export let beautyEnabled = false;
	export let beautyVotesPerPlayer = 2;
	export let beautyVotesPerPlayerMax = 1;
	export let beautyAllowDuplicateVotes = false;
	export let beautySplitPointsOnTie = true;
	export let beautyPointsBonus = 2;
	export let beautyResultsDisplayMode: BeautyResultsDisplayMode = 'combined';

	$: effectiveVotesPerGuesser = Math.max(
		1,
		Math.min(votesPerGuesser, Math.max(votesPerGuesserMax, 1))
	);
	$: storytellerLabel = activePlayer || 'Storyteller';
	$: storytellerWinCondition = Math.max(1, storytellerLossComplement + 1);
	$: effectiveDoubleVoteBonusTooManyWrongPoints = doubleVoteBonusTooManyWrongFollowsNormal
		? doubleVoteBonusNormalPoints
		: doubleVoteBonusTooManyWrongPoints;
	$: effectiveDoubleVoteBonusTooManyCorrectPoints = doubleVoteBonusTooManyCorrectFollowsNormal
		? doubleVoteBonusNormalPoints
		: doubleVoteBonusTooManyCorrectPoints;
	$: effectiveBeautyVotesPerPlayer = Math.max(
		1,
		Math.min(beautyVotesPerPlayer, Math.max(beautyVotesPerPlayerMax, 1))
	);
	$: formattedEffectiveBeautyDivisor =
		$beautyVotePointsDivisorEffective === null
			? 'pending'
			: $beautyVotePointsDivisorEffective.toFixed(1);

	function formatDoubleVoteBonus(points: number) {
		return points === 0 ? 'off' : `+${points}`;
	}
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
			Storyteller wins when at least {storytellerWinCondition} guesser{storytellerWinCondition === 1
				? ''
				: 's'} are different from the rest.
		</p>
		<ul class="ml-5 list-disc space-y-1 text-sm">
			<li>
				Storyteller-loss round: <span class="font-semibold">{storytellerLabel}</span> +0, each guesser
				+2.
			</li>
			{#if bonusThresholdLossTogglesApplyToAllStorytellerLossRounds}
				<li>The correct-guess bonus below applies in any storyteller-loss round.</li>
			{:else}
				<li>The correct-guess bonus below applies only when enough guessers were correct.</li>
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
				Normal round: <span class="font-semibold">{storytellerLabel}</span>
				+{storytellerSuccessPoints}, each guesser with at least one correct vote +3.
			</li>
			{#if doubleVoteBonusTooManyWrongFollowsNormal && doubleVoteBonusTooManyCorrectFollowsNormal}
				<li>
					Double-correct bonus: {formatDoubleVoteBonus(doubleVoteBonusNormalPoints)} if 2+ of your
					{effectiveVotesPerGuesser} vote{effectiveVotesPerGuesser === 1 ? '' : 's'} hit storyteller
					card.
				</li>
			{:else}
				<li>
					Double-correct bonus when 2+ of your {effectiveVotesPerGuesser} vote
					{effectiveVotesPerGuesser === 1 ? '' : 's'} hit storyteller card: normal
					{formatDoubleVoteBonus(doubleVoteBonusNormalPoints)}, too many guessed wrong
					{formatDoubleVoteBonus(effectiveDoubleVoteBonusTooManyWrongPoints)}, too many guessed
					correctly {formatDoubleVoteBonus(effectiveDoubleVoteBonusTooManyCorrectPoints)}.
				</li>
			{/if}
			<li>Decoy bonus: +1 per vote token on your card (max +3, non-storyteller only).</li>
			{#if beautyEnabled}
				<li>
					Most Beautiful: every active player casts {effectiveBeautyVotesPerPlayer} beauty vote
					{effectiveBeautyVotesPerPlayer === 1 ? '' : 's'} on other players’ cards.
				</li>
				{#if $beautyScoringMode === 'vote_divisor'}
					<li>
						Beauty scoring: each owner gets floor(cumulative current-game beauty votes on their
						submitted cards / K).
					</li>
					<li>
						K mode: {$beautyVotePointsDivisorMode}. Effective K:
						{formattedEffectiveBeautyDivisor === 'pending'
							? 'pending first beauty results.'
							: formattedEffectiveBeautyDivisor + '.'}
					</li>
				{:else}
					<li>
						Beauty winners: each top-voted owner gets +{beautyPointsBonus}
						{beautyPointsBonus === 1 ? ' point' : ' points'}
						{beautySplitPointsOnTie
							? ' split among tied owners with rounding up when needed.'
							: ' once per tied owner.'}
					</li>
				{/if}
				<li>
					Beauty duplicates: {beautyAllowDuplicateVotes ? 'allowed' : 'not allowed'}. Results combine storyteller and beauty votes.
				</li>
			{/if}
			{#if $clueRatingEnabled}
				<li>
					Clue stars: non-storytellers rate the clue from 1 to {$clueRatingMaxStars} star{$clueRatingMaxStars ===
					1
						? ''
						: 's'}.
				</li>
				<li>Storyteller clue bonus: max(round(average stars) - 1, 0).</li>
				<li>Skipped or timed-out raters are excluded from the average.</li>
			{/if}
		</ul>
	{/if}
</div>
