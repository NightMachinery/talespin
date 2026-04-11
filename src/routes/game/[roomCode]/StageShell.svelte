<script lang="ts">
	import { onMount } from 'svelte';
	import { cardsFitToHeight } from '$lib/viewOptions';
	import type GameServer from '$lib/gameServer';
	import Leaderboard from './Leaderboard.svelte';
	import MostBeautifulStatsPanel from './MostBeautifulStatsPanel.svelte';
	import SidebarOptions from './SidebarOptions.svelte';
	import ScoreCheatsheet from './ScoreCheatsheet.svelte';
	import type {
		BeautyResultsDisplayMode,
		GameMode,
		ObserverInfo,
		PlayerInfo,
		StellaQueuedRevealMode,
		WinCondition
	} from '$lib/types';

	export let players: { [key: string]: PlayerInfo } = {};
	export let observers: { [key: string]: ObserverInfo } = {};
	export let name = '';
	export let creator = '';
	export let moderators: string[] = [];
	export let gameServer: GameServer;
	export let stage = '';
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
	export let beautySplitPointsOnTie = true;
	export let beautyPointsBonus = 2;
	export let beautyPointsBonusMin = 0;
	export let beautyPointsBonusMax = 10;
	export let beautyResultsDisplayMode: BeautyResultsDisplayMode = 'combined';
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
	export let forceHintChoosingTimer = false;
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
	export let stellaQueuedRevealMode: StellaQueuedRevealMode = 'animated';
	export let stellaScoutTimerEnabled = true;
	export let stellaScoutTimerDurationS = 10;
	export let forceStellaScoutTimer = false;
	export let serverTimeMs: number | null = null;
	export let currentStageDeadlineS: number | null = null;
	export let votingWrongCardDisableDistribution: number[] = [1];
	export let activePlayer = '';
	export let pointChange: { [key: string]: number } = {};
	export let roundNum = 0;
	export let cardsRemaining = 0;
	export let deckRefillFlashToken = 0;
	export let darkPlayer = '';
	export let winCondition: WinCondition = {
		mode: 'points',
		target_points: 10
	};
	export let gameMode: GameMode = 'dixit_plus';
	export let showMobileActions = true;

	const hasMobileTop = !!$$slots.mobileTop;
	const hasMobileActions = !!$$slots.mobileActions;
	const hasMobileBottom = !!$$slots.mobileBottom;
	const hasSidebarBottom = !!$$slots.sidebarBottom;
	let nowPerfMs = 0;
	let timerInterval: number | undefined;
	let lastServerTimeMs: number | null = null;
	let syncedServerTimeMs: number | null = null;
	let syncedAtPerfMs = 0;

	onMount(() => {
		nowPerfMs = performance.now();
		timerInterval = window.setInterval(() => {
			nowPerfMs = performance.now();
		}, 250);

		return () => {
			if (timerInterval) {
				window.clearInterval(timerInterval);
			}
		};
	});

	function formatCountdown(totalSeconds: number) {
		const minutes = Math.floor(totalSeconds / 60);
		const seconds = totalSeconds % 60;
		return `${minutes}:${seconds.toString().padStart(2, '0')}`;
	}

	$: showMobileOptions = true;
	$: mainContentClass = `rounded-lg bg-black/10 p-2 sm:p-3 lg:p-4 ${
		$cardsFitToHeight ? 'lg:h-full' : ''
	}`;
	$: if (serverTimeMs !== lastServerTimeMs) {
		lastServerTimeMs = serverTimeMs;
		syncedServerTimeMs = serverTimeMs;
		syncedAtPerfMs = nowPerfMs;
	}
	$: hasStageTimer = currentStageDeadlineS !== null;
	$: estimatedServerNowMs =
		syncedServerTimeMs === null ? null : syncedServerTimeMs + (nowPerfMs - syncedAtPerfMs);
	$: remainingStageTimerSeconds =
		currentStageDeadlineS === null || estimatedServerNowMs === null
			? 0
			: Math.max(0, Math.ceil((currentStageDeadlineS * 1000 - estimatedServerNowMs) / 1000));
	$: stageTimerLabel =
		stage === 'ActiveChooses'
			? 'Hint timer'
			: stage === 'StellaAssociate'
				? 'Associate timer'
				: stage === 'PlayersChoose'
					? 'Card choosing timer'
					: stage === 'StellaReveal'
						? 'Reveal timer'
						: stage === 'Voting'
							? 'Voting timer'
							: stage === 'BeautyVoting'
								? 'Beauty timer'
								: 'Stage timer';
	$: stageTimerDisplay = formatCountdown(remainingStageTimerSeconds);
	$: stageTimerExpired = hasStageTimer && remainingStageTimerSeconds === 0;
</script>

<div class="w-full px-3 pt-3 lg:px-6 lg:pt-4">
	<div class="mx-auto max-w-[1400px]">
		{#if hasMobileTop}
			<div class="mb-3 lg:hidden">
				<slot name="mobileTop" />
			</div>
		{/if}

		{#if hasStageTimer}
			<div class="mb-3">
				<div
					class={`card flex items-center justify-between gap-3 px-4 py-3 ${
						stageTimerExpired ? 'bg-warning-900/55 ring-1 ring-warning-400/45' : 'light'
					}`}
				>
					<div>
						<p class="text-xs font-semibold uppercase tracking-wide opacity-70">
							{stageTimerLabel}
						</p>
						<p class="text-sm opacity-85">
							{#if stageTimerExpired}
								Time is up.
							{:else}
								Shared countdown for this stage.
							{/if}
						</p>
					</div>
					<div
						class={`rounded px-3 py-1 text-2xl font-bold tabular-nums ${
							stageTimerExpired ? 'bg-warning-500 text-black' : 'bg-black/30 text-white'
						}`}
					>
						{stageTimerDisplay}
					</div>
				</div>
			</div>
		{/if}

		<div class="flex flex-col gap-4 lg:h-[calc(100vh-2rem)] lg:flex-row">
			<main class="order-1 min-h-[58vh] flex-1 lg:order-2 lg:min-h-0">
				<div class={mainContentClass}>
					<slot />
				</div>
			</main>

			{#if hasMobileActions && showMobileActions}
				<div class="order-2 sticky bottom-2 z-20 lg:hidden">
					<div class="card light p-3">
						<slot name="mobileActions" />
					</div>
				</div>
			{/if}

			<aside class="order-3 w-full lg:order-1 lg:w-[340px] lg:shrink-0">
				<div
					class="flex flex-col gap-4 lg:sticky lg:top-4 lg:max-h-[calc(100vh-2rem)] lg:overflow-y-auto lg:pr-1"
				>
					<Leaderboard
						{players}
						{observers}
						{name}
						{moderators}
						{gameServer}
						{stage}
						{pointChange}
						{activePlayer}
						{roundNum}
						{cardsRemaining}
						{deckRefillFlashToken}
						{darkPlayer}
						{winCondition}
						{gameMode}
					/>

					<div class="hidden lg:block">
						<slot name="leftRail" />
					</div>

					{#if hasMobileBottom}
						<div class="lg:hidden">
							<slot name="mobileBottom" />
						</div>
					{/if}

					{#if showMobileOptions}
						<div class="lg:hidden">
							<SidebarOptions
								{players}
								{observers}
								{name}
								{creator}
								{moderators}
								{gameServer}
								{stage}
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
								{beautySplitPointsOnTie}
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
								{forceHintChoosingTimer}
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
								{votingWrongCardDisableDistribution}
								{activePlayer}
								{gameMode}
							/>
						</div>
					{/if}

					<div class="hidden lg:block">
						<SidebarOptions
							{players}
							{observers}
							{name}
							{creator}
							{moderators}
							{gameServer}
							{stage}
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
							{beautySplitPointsOnTie}
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
							{forceHintChoosingTimer}
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
							{votingWrongCardDisableDistribution}
							{activePlayer}
							{gameMode}
						/>
					</div>

					{#if gameMode === 'dixit_plus'}
						<MostBeautifulStatsPanel title="Most Beautiful ranking" compact />
					{/if}

					{#if hasSidebarBottom}
						<div>
							<slot name="sidebarBottom" />
						</div>
					{/if}

					<div>
						<ScoreCheatsheet
							{gameMode}
							{activePlayer}
							{storytellerLossComplement}
							{votesPerGuesser}
							{votesPerGuesserMax}
							{beautyEnabled}
							{beautyVotesPerPlayer}
							{beautyVotesPerPlayerMax}
							{beautyAllowDuplicateVotes}
							{beautySplitPointsOnTie}
							{beautyPointsBonus}
							{beautyResultsDisplayMode}
							{bonusCorrectGuessOnThresholdCorrectLoss}
							{bonusDoubleVoteOnThresholdCorrectLoss}
							{bonusThresholdLossTogglesApplyToAllStorytellerLossRounds}
						/>
					</div>
				</div>
			</aside>
		</div>
	</div>
</div>
