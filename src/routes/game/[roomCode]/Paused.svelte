<script lang="ts">
	import type GameServer from '$lib/gameServer';
	import type { GameMode, ObserverInfo, PlayerInfo, WinCondition } from '$lib/types';
	import StageShell from './StageShell.svelte';

	export let name = '';
	export let creator = '';
	export let moderators: string[] = [];
	export let players: { [key: string]: PlayerInfo } = {};
	export let observers: { [key: string]: ObserverInfo } = {};
	export let gameServer: GameServer;
	export let reason = '';
	export let allowNewPlayersMidgame = true;
	export let storytellerLossComplement = 0;
	export let storytellerLossComplementMin = 0;
	export let storytellerLossComplementMax = 0;
	export let storytellerLossComplementAuto = true;
	export let votesPerGuesser = 1;
	export let votesPerGuesserMin = 1;
	export let votesPerGuesserMax = 1;
	export let beautyEnabled = false;
	export let beautyVotesPerPlayer = 1;
	export let beautyVotesPerPlayerMin = 1;
	export let beautyVotesPerPlayerMax = 1;
	export let beautyAllowDuplicateVotes = false;
	export let beautyPointsBonus = 2;
	export let beautyPointsBonusMin = 0;
	export let beautyPointsBonusMax = 10;
	export let beautyResultsDisplayMode: import('$lib/types').BeautyResultsDisplayMode = 'combined';
	export let cardsPerHand = 12;
	export let cardsPerHandMin = 1;
	export let cardsPerHandMax = 18;
	export let nominationsPerGuesser = 1;
	export let nominationsPerGuesserMin = 1;
	export let nominationsPerGuesserMax = 1;
	export let bonusCorrectGuessOnThresholdCorrectLoss = true;
	export let bonusDoubleVoteOnThresholdCorrectLoss = true;
	export let bonusThresholdLossTogglesApplyToAllStorytellerLossRounds = true;
	export let showVotingCardNumbers = true;
	export let roundStartDiscardCount = 3;
	export let hintChoosingTimerEnabled = true;
	export let hintChoosingTimerDurationS = 60;
	export let cardChoosingTimerEnabled = true;
	export let cardChoosingTimerDurationS = 30;
	export let votingTimerEnabled = true;
	export let votingTimerDurationS = 180;
	export let beautyTimerEnabled = true;
	export let beautyTimerDurationS = 60;
	export let forceCardChoosingTimer = false;
	export let forceVotingTimer = false;
	export let forceBeautyTimer = false;
	export let stellaBoardSize = 15;
	export let stellaBoardSizeMin = 1;
	export let stellaBoardSizeMax = 100;
	export let stellaSelectionMin = 1;
	export let stellaSelectionMax = 10;
	export let stellaSelectionCountMin = 1;
	export let stellaSelectionCountMax = 15;
	export let stellaWordPackPresetNames: string[] = [];
	export let stellaSelectedWordPackName = '';
	export let stellaWordPackIsUnsaved = false;
	export let stellaQueueDuringAssociation = true;
	export let stellaQueuedRevealMode: 'animated' | 'fast' = 'animated';
	export let stellaScoutTimerEnabled = true;
	export let stellaScoutTimerDurationS = 10;
	export let forceStellaScoutTimer = false;
	export let serverTimeMs: number | null = null;
	export let currentStageDeadlineS: number | null = null;
	export let votingWrongCardDisableDistribution: number[] = [1];
	export let roundNum = 0;
	export let cardsRemaining = 0;
	export let deckRefillFlashToken = 0;
	export let winCondition: WinCondition = {
		mode: 'points',
		target_points: 10
	};
	export let gameMode: GameMode = 'dixit_plus';

	$: moderatorSet = new Set(moderators);
	$: isModerator = moderatorSet.has(name);
	$: nonObserverPlayerCount = Object.keys(players).length;
	$: canResume = nonObserverPlayerCount >= 3;
</script>

<StageShell
	{players}
	{observers}
	{name}
	{creator}
	{moderators}
	{gameServer}
	stage="Paused"
	{allowNewPlayersMidgame}
	{storytellerLossComplement}
	{storytellerLossComplementMin}
	{storytellerLossComplementMax}
	{storytellerLossComplementAuto}
	{votesPerGuesser}
	{votesPerGuesserMin}
	{votesPerGuesserMax}
	{beautyEnabled}
	{beautyVotesPerPlayer}
	{beautyVotesPerPlayerMin}
	{beautyVotesPerPlayerMax}
	{beautyAllowDuplicateVotes}
	{beautyPointsBonus}
	{beautyPointsBonusMin}
	{beautyPointsBonusMax}
	{beautyResultsDisplayMode}
	{cardsPerHand}
	{cardsPerHandMin}
	{cardsPerHandMax}
	{nominationsPerGuesser}
	{nominationsPerGuesserMin}
	{nominationsPerGuesserMax}
	{bonusCorrectGuessOnThresholdCorrectLoss}
	{bonusDoubleVoteOnThresholdCorrectLoss}
	{bonusThresholdLossTogglesApplyToAllStorytellerLossRounds}
	{showVotingCardNumbers}
	{roundStartDiscardCount}
	{hintChoosingTimerEnabled}
	{hintChoosingTimerDurationS}
	{cardChoosingTimerEnabled}
	{cardChoosingTimerDurationS}
	{votingTimerEnabled}
	{votingTimerDurationS}
	{beautyTimerEnabled}
	{beautyTimerDurationS}
	{forceCardChoosingTimer}
	{forceVotingTimer}
	{forceBeautyTimer}
	{stellaBoardSize}
	{stellaBoardSizeMin}
	{stellaBoardSizeMax}
	{stellaSelectionMin}
	{stellaSelectionMax}
	{stellaSelectionCountMin}
	{stellaSelectionCountMax}
	{stellaWordPackPresetNames}
	{stellaSelectedWordPackName}
	{stellaWordPackIsUnsaved}
	{stellaQueueDuringAssociation}
	{stellaQueuedRevealMode}
	{stellaScoutTimerEnabled}
	{stellaScoutTimerDurationS}
	{forceStellaScoutTimer}
	{serverTimeMs}
	{currentStageDeadlineS}
	{votingWrongCardDisableDistribution}
	activePlayer=""
	pointChange={{}}
	{roundNum}
	{cardsRemaining}
	{deckRefillFlashToken}
	{winCondition}
	{gameMode}
	showMobileActions={isModerator}
>
	<svelte:fragment slot="leftRail">
		<div class="card light space-y-2 p-4">
			<h1 class="text-xl font-semibold">Game paused</h1>
			<p>{reason || 'Need at least 3 non-observer players to continue.'}</p>
		</div>
		{#if isModerator}
			<div class="card light p-4">
				<button
					class="btn variant-filled w-full"
					disabled={!canResume}
					on:click={() => gameServer.resumeGame()}
				>
					Resume Game
				</button>
				{#if !canResume}
					<p class="mt-2 text-xs opacity-70">Need at least 3 non-observer players to resume.</p>
				{/if}
			</div>
		{/if}
	</svelte:fragment>

	<svelte:fragment slot="mobileTop">
		<div class="card light space-y-2 p-4">
			<h1 class="text-xl font-semibold">Game paused</h1>
			<p>{reason || 'Need at least 3 non-observer players to continue.'}</p>
		</div>
	</svelte:fragment>

	<svelte:fragment slot="mobileActions">
		{#if isModerator}
			<button
				class="btn variant-filled w-full"
				disabled={!canResume}
				on:click={() => gameServer.resumeGame()}
			>
				Resume Game
			</button>
		{/if}
	</svelte:fragment>

	<div class="card light p-6">
		<p>Waiting for enough non-observer players.</p>
	</div>
</StageShell>
