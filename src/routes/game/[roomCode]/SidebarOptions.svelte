<script lang="ts">
	import { browser } from '$app/environment';
	import type GameServer from '$lib/gameServer';
	import type { ObserverInfo, PlayerInfo } from '$lib/types';
	import { OFFLINE_STATUS_LABEL } from '$lib/presence';
	import { cardsFitToHeight } from '$lib/viewOptions';
	import {
		DEFAULT_VOTING_WRONG_CARD_DISABLE_DISTRIBUTION,
		MAX_VOTING_WRONG_CARD_DISABLE_X,
		VOTING_WRONG_CARD_DISABLE_PRESETS,
		areVotingWrongCardDisableDistributionsEqual,
		findVotingWrongCardDisablePresetId,
		getVotingWrongCardDisablePreset,
		normalizeVotingWrongCardDisableDistribution,
		resizeVotingWrongCardDisableDistribution,
		setVotingWrongCardDisableProbability,
		type VotingWrongCardDisablePresetId
	} from '$lib/votingWrongCardDisableDistribution';

	const SETTINGS_EDIT_STAGE_HINT = 'Can only be changed during storyteller choosing stage.';
	const STAGE_TIMER_DURATION_MIN_S = 1;
	const STAGE_TIMER_DURATION_MAX_S = 60 * 60;

	export let players: { [key: string]: PlayerInfo } = {};
	export let observers: { [key: string]: ObserverInfo } = {};
	export let name = '';
	export let creator = '';
	export let moderators: string[] = [];
	export let stage = '';
	export let activePlayer = '';
	export let allowNewPlayersMidgame = true;
	export let storytellerLossComplement = 0;
	export let storytellerLossComplementMin = 0;
	export let storytellerLossComplementMax = 0;
	export let storytellerLossComplementAuto = true;
	export let votesPerGuesser = 1;
	export let votesPerGuesserMin = 1;
	export let votesPerGuesserMax = 1;
	export let cardsPerHand = 12;
	export let cardsPerHandMin = 1;
	export let cardsPerHandMax = 18;
	export let nominationsPerGuesser = 1;
	export let nominationsPerGuesserMin = 1;
	export let nominationsPerGuesserMax = 1;
	export let bonusCorrectGuessOnThresholdCorrectLoss = true;
	export let bonusDoubleVoteOnThresholdCorrectLoss = true;
	export let showVotingCardNumbers = true;
	export let roundStartDiscardCount = 3;
	export let hintChoosingTimerEnabled = true;
	export let hintChoosingTimerDurationS = 60;
	export let cardChoosingTimerEnabled = true;
	export let cardChoosingTimerDurationS = 30;
	export let votingTimerEnabled = true;
	export let votingTimerDurationS = 180;
	export let forceCardChoosingTimer = false;
	export let forceVotingTimer = false;
	export let votingWrongCardDisableDistribution: number[] = [
		...DEFAULT_VOTING_WRONG_CARD_DISABLE_DISTRIBUTION
	];
	export let gameServer: GameServer;

	$: moderatorSet = new Set(moderators);
	$: sortedPlayerEntries = Object.entries(players).sort(([a], [b]) => a.localeCompare(b));
	$: sortedObserverEntries = Object.entries(observers).sort(([a], [b]) => a.localeCompare(b));
	$: isCreator = creator !== '' && creator === name;
	$: isModerator = moderatorSet.has(name);
	$: showModeration = stage !== 'End' && (isCreator || isModerator);
	$: isSelfObserver = !!observers[name];
	$: selfObserverInfo = observers[name];
	$: selfObserveBlocked =
		(stage === 'PlayersChoose' || stage === 'Voting') && activePlayer === name;
	$: canBecomeObserver =
		!isSelfObserver && stage !== 'Joining' && stage !== 'End' && !selfObserveBlocked;
	$: canChangeCardsPerHand = stage === 'ActiveChooses';
	$: canChangePreVotingSettings = stage === 'ActiveChooses';
	$: canRefreshHands = stage === 'ActiveChooses';
	$: storytellerWinCondition = storytellerLossComplement + 1;
	$: storytellerWinConditionMin = storytellerLossComplementMin + 1;
	$: storytellerWinConditionMax = storytellerLossComplementMax + 1;
	$: roundStartDiscardCountMax = Math.max(0, cardsPerHand - 1);
	$: normalizedVotingWrongCardDisableDistribution = normalizeVotingWrongCardDisableDistribution(
		votingWrongCardDisableDistribution
	);
	$: selectedVotingWrongCardDisablePresetId = findVotingWrongCardDisablePresetId(
		normalizedVotingWrongCardDisableDistribution
	);
	$: votingWrongCardDisableMax = Math.max(
		0,
		normalizedVotingWrongCardDisableDistribution.length - 1
	);
	$: selfJoinPending =
		!!selfObserverInfo &&
		(selfObserverInfo.join_requested || selfObserverInfo.auto_join_on_next_round);
	$: selfJoinBackLabel =
		stage === 'Voting' ? (selfJoinPending ? 'Cancel pending join' : 'Join next round') : 'Join now';

	function isPlayerModerator(playerName: string) {
		return moderatorSet.has(playerName);
	}

	function kickPlayer(playerName: string) {
		if (!isModerator || playerName === name) return;
		if (!browser || window.confirm(`Kick ${playerName} from this game?`)) {
			gameServer.kickPlayer(playerName);
		}
	}

	function setModerator(playerName: string, enabled: boolean) {
		if (playerName === creator) return;
		if (enabled) {
			if (!isCreator && !isModerator) return;
		} else if (!isCreator) {
			return;
		}
		gameServer.setModerator(playerName, enabled);
	}

	function setObserver(playerName: string, enabled: boolean) {
		gameServer.setObserver(playerName, enabled);
	}

	function becomeObserver() {
		if (!canBecomeObserver) return;
		if (!browser || window.confirm('Switch to observer mode?')) {
			gameServer.setObserver(name, true);
		}
	}

	function joinBack() {
		gameServer.requestJoinFromObserver();
	}

	function observerJoinActionLabel(observerInfo: ObserverInfo) {
		if (stage !== 'Voting') return 'Join now';
		const pending = observerInfo.join_requested || observerInfo.auto_join_on_next_round;
		return pending ? 'Cancel pending join' : 'Join next round';
	}

	function updateAllowMidgameJoin(event: Event) {
		const input = event.currentTarget as HTMLInputElement;
		gameServer.setAllowMidgameJoin(input.checked);
	}

	function updateStorytellerLossComplement(event: Event) {
		const input = event.currentTarget as HTMLInputElement;
		const parsed = Number(input.value);
		if (!isModerator || storytellerLossComplementAuto) {
			input.value = `${storytellerWinCondition}`;
			return;
		}
		if (
			!Number.isInteger(parsed) ||
			parsed < storytellerWinConditionMin ||
			parsed > storytellerWinConditionMax
		) {
			input.value = `${storytellerWinCondition}`;
			return;
		}
		const complement = parsed - 1;
		if (complement !== storytellerLossComplement) {
			gameServer.setStorytellerLossComplement(complement);
		}
	}

	function updateStorytellerLossComplementAuto(event: Event) {
		const input = event.currentTarget as HTMLInputElement;
		if (!isModerator) {
			input.checked = storytellerLossComplementAuto;
			return;
		}
		gameServer.setStorytellerLossComplementAuto(input.checked);
	}

	function updateVotesPerGuesser(event: Event) {
		const input = event.currentTarget as HTMLInputElement;
		const parsed = Number(input.value);
		if (!Number.isInteger(parsed) || parsed < votesPerGuesserMin || parsed > votesPerGuesserMax) {
			input.value = `${votesPerGuesser}`;
			return;
		}
		if (parsed !== votesPerGuesser) {
			gameServer.setVotesPerGuesser(parsed);
		}
	}

	function updateCardsPerHand(event: Event) {
		const input = event.currentTarget as HTMLInputElement;
		const parsed = Number(input.value);
		if (!Number.isInteger(parsed) || parsed < cardsPerHandMin || parsed > cardsPerHandMax) {
			input.value = `${cardsPerHand}`;
			return;
		}
		if (parsed !== cardsPerHand) {
			gameServer.setCardsPerHand(parsed);
		}
	}

	function updateNominationsPerGuesser(event: Event) {
		const input = event.currentTarget as HTMLInputElement;
		const parsed = Number(input.value);
		if (
			!Number.isInteger(parsed) ||
			parsed < nominationsPerGuesserMin ||
			parsed > nominationsPerGuesserMax
		) {
			input.value = `${nominationsPerGuesser}`;
			return;
		}
		if (parsed !== nominationsPerGuesser) {
			gameServer.setNominationsPerGuesser(parsed);
		}
	}

	function updateBonusCorrectGuessOnThresholdCorrectLoss(event: Event) {
		const input = event.currentTarget as HTMLInputElement;
		if (!isModerator || !canChangePreVotingSettings) {
			input.checked = bonusCorrectGuessOnThresholdCorrectLoss;
			return;
		}
		gameServer.setBonusCorrectGuessOnThresholdCorrectLoss(input.checked);
	}

	function updateBonusDoubleVoteOnThresholdCorrectLoss(event: Event) {
		const input = event.currentTarget as HTMLInputElement;
		if (!isModerator || !canChangePreVotingSettings) {
			input.checked = bonusDoubleVoteOnThresholdCorrectLoss;
			return;
		}
		gameServer.setBonusDoubleVoteOnThresholdCorrectLoss(input.checked);
	}

	function updateShowVotingCardNumbers(event: Event) {
		const input = event.currentTarget as HTMLInputElement;
		if (!isModerator || !canChangePreVotingSettings) {
			input.checked = showVotingCardNumbers;
			return;
		}
		gameServer.setShowVotingCardNumbers(input.checked);
	}

	function updateRoundStartDiscardCount(event: Event) {
		const input = event.currentTarget as HTMLInputElement;
		const parsed = Number(input.value);
		if (!Number.isInteger(parsed) || parsed < 0 || parsed > roundStartDiscardCountMax) {
			input.value = `${roundStartDiscardCount}`;
			return;
		}
		if (parsed !== roundStartDiscardCount) {
			gameServer.setRoundStartDiscardCount(parsed);
		}
	}

	function updateStageTimerToggle(
		event: Event,
		currentValue: boolean,
		setter: (enabled: boolean) => void
	) {
		const input = event.currentTarget as HTMLInputElement;
		if (!isModerator || !canChangePreVotingSettings) {
			input.checked = currentValue;
			return;
		}
		setter(input.checked);
	}

	function updateStageTimerDuration(
		event: Event,
		currentValue: number,
		setter: (seconds: number) => void
	) {
		const input = event.currentTarget as HTMLInputElement;
		const parsed = Number(input.value);
		if (!isModerator || !canChangePreVotingSettings) {
			input.value = `${currentValue}`;
			return;
		}
		if (
			!Number.isInteger(parsed) ||
			parsed < STAGE_TIMER_DURATION_MIN_S ||
			parsed > STAGE_TIMER_DURATION_MAX_S
		) {
			input.value = `${currentValue}`;
			return;
		}
		if (parsed !== currentValue) {
			setter(parsed);
		}
	}

	function updateHintChoosingTimerEnabled(event: Event) {
		updateStageTimerToggle(event, hintChoosingTimerEnabled, (enabled) =>
			gameServer.setHintChoosingTimerEnabled(enabled)
		);
	}

	function updateHintChoosingTimerDuration(event: Event) {
		updateStageTimerDuration(event, hintChoosingTimerDurationS, (seconds) =>
			gameServer.setHintChoosingTimerDuration(seconds)
		);
	}

	function updateCardChoosingTimerEnabled(event: Event) {
		updateStageTimerToggle(event, cardChoosingTimerEnabled, (enabled) =>
			gameServer.setCardChoosingTimerEnabled(enabled)
		);
	}

	function updateCardChoosingTimerDuration(event: Event) {
		updateStageTimerDuration(event, cardChoosingTimerDurationS, (seconds) =>
			gameServer.setCardChoosingTimerDuration(seconds)
		);
	}

	function updateVotingTimerEnabled(event: Event) {
		updateStageTimerToggle(event, votingTimerEnabled, (enabled) =>
			gameServer.setVotingTimerEnabled(enabled)
		);
	}

	function updateVotingTimerDuration(event: Event) {
		updateStageTimerDuration(event, votingTimerDurationS, (seconds) =>
			gameServer.setVotingTimerDuration(seconds)
		);
	}

	function updateForceCardChoosingTimer(event: Event) {
		updateStageTimerToggle(event, forceCardChoosingTimer, (enabled) =>
			gameServer.setForceCardChoosingTimer(enabled)
		);
	}

	function updateForceVotingTimer(event: Event) {
		updateStageTimerToggle(event, forceVotingTimer, (enabled) =>
			gameServer.setForceVotingTimer(enabled)
		);
	}

	function formatPercent(probability: number) {
		return `${(probability * 100).toFixed(1).replace(/\\.0$/, '')}%`;
	}

	function updateVotingWrongCardDisablePreset(event: Event) {
		const select = event.currentTarget as HTMLSelectElement;
		if (!isModerator || !canChangePreVotingSettings) {
			select.value = selectedVotingWrongCardDisablePresetId;
			return;
		}

		const presetId = select.value as VotingWrongCardDisablePresetId;
		if (presetId === 'custom') {
			select.value = selectedVotingWrongCardDisablePresetId;
			return;
		}

		const preset = getVotingWrongCardDisablePreset(presetId);
		if (!preset) {
			select.value = selectedVotingWrongCardDisablePresetId;
			return;
		}

		if (
			!areVotingWrongCardDisableDistributionsEqual(
				preset.distribution,
				normalizedVotingWrongCardDisableDistribution
			)
		) {
			gameServer.setVotingWrongCardDisableDistribution(preset.distribution);
		}
	}

	function updateVotingWrongCardDisableMax(event: Event) {
		const input = event.currentTarget as HTMLInputElement;
		const parsed = Number(input.value);
		if (!isModerator || !canChangePreVotingSettings) {
			input.value = `${votingWrongCardDisableMax}`;
			return;
		}

		if (!Number.isInteger(parsed) || parsed < 0 || parsed > MAX_VOTING_WRONG_CARD_DISABLE_X) {
			input.value = `${votingWrongCardDisableMax}`;
			return;
		}

		const nextDistribution = resizeVotingWrongCardDisableDistribution(
			normalizedVotingWrongCardDisableDistribution,
			parsed
		);
		if (
			!areVotingWrongCardDisableDistributionsEqual(
				nextDistribution,
				normalizedVotingWrongCardDisableDistribution
			)
		) {
			gameServer.setVotingWrongCardDisableDistribution(nextDistribution);
		}
	}

	function updateVotingWrongCardDisableProbability(index: number, nextPercent: number) {
		if (!isModerator || !canChangePreVotingSettings) return;

		const sanitizedPercent = Math.min(100, Math.max(0, Math.round(nextPercent)));
		const nextDistribution = setVotingWrongCardDisableProbability(
			normalizedVotingWrongCardDisableDistribution,
			index,
			sanitizedPercent / 100
		);
		if (
			!areVotingWrongCardDisableDistributionsEqual(
				nextDistribution,
				normalizedVotingWrongCardDisableDistribution
			)
		) {
			gameServer.setVotingWrongCardDisableDistribution(nextDistribution);
		}
	}

	function updateVotingWrongCardDisableProbabilityFromEvent(index: number, event: Event) {
		const input = event.currentTarget as HTMLInputElement;
		updateVotingWrongCardDisableProbability(index, Number(input.value));
	}

	function refreshHands() {
		if (!isModerator || !canRefreshHands) return;
		if (!browser || window.confirm('Discard and redraw all active player hands now?')) {
			gameServer.refreshHands();
		}
	}
</script>

<div class="card light space-y-3 p-4">
	<h2 class="text-lg font-semibold">Options</h2>
	<label class="flex items-start gap-3">
		<input
			type="checkbox"
			class="mt-1 h-4 w-4 cursor-pointer accent-primary-500"
			bind:checked={$cardsFitToHeight}
		/>
		<span class="block font-medium">Cards fit to height</span>
	</label>
	{#if stage !== 'Joining' && stage !== 'End'}
		{#if isSelfObserver}
			<button class="btn variant-filled w-full" on:click={joinBack}>{selfJoinBackLabel}</button>
		{:else}
			<button
				class="btn variant-filled w-full"
				on:click={becomeObserver}
				disabled={!canBecomeObserver}
			>
				Become observer
			</button>
			{#if !canBecomeObserver && selfObserveBlocked}
				<p class="text-xs opacity-70">
					Storyteller can only become observer before choosing card and clue.
				</p>
			{/if}
		{/if}
	{/if}
	{#if showModeration}
		<details class="rounded border border-white/20 px-3 py-2">
			<summary class="cursor-pointer text-sm font-semibold">Moderation</summary>
			<div class="mt-3 max-h-[45vh] space-y-2 overflow-y-auto pr-1">
				<p class="text-xs font-semibold uppercase tracking-wide opacity-70">Manage players</p>
				{#each sortedPlayerEntries as [playerName]}
					<div class="rounded border border-white/20 px-2 py-1.5">
						<div class="flex items-start justify-between gap-2">
							<div class="min-w-0 break-words font-semibold">
								{playerName}
								{#if playerName === creator}
									<span class="ml-1 text-xs font-normal opacity-70">(creator)</span>
								{:else if isPlayerModerator(playerName)}
									<span class="ml-1 text-xs font-normal opacity-70">(mod)</span>
								{/if}
							</div>
							<div class="flex flex-wrap justify-end gap-1.5">
								{#if isModerator && playerName !== name}
									<button
										class="btn variant-filled shrink-0 px-2 py-0.5 text-xs"
										on:click={() => kickPlayer(playerName)}
									>
										Kick
									</button>
								{/if}
								{#if playerName !== creator}
									{#if isCreator && isPlayerModerator(playerName)}
										<button
											class="btn variant-filled shrink-0 px-2 py-0.5 text-xs"
											on:click={() => setModerator(playerName, false)}
										>
											Demote
										</button>
									{:else if (isCreator || isModerator) && !isPlayerModerator(playerName)}
										<button
											class="btn variant-filled shrink-0 px-2 py-0.5 text-xs"
											on:click={() => setModerator(playerName, true)}
										>
											Make Mod
										</button>
									{/if}
								{/if}
								{#if isModerator || playerName === name}
									<button
										class="btn variant-filled shrink-0 px-2 py-0.5 text-xs"
										on:click={() => setObserver(playerName, true)}
									>
										Observer
									</button>
								{/if}
							</div>
						</div>
					</div>
				{/each}
				{#each sortedObserverEntries as [observerName, info]}
					<div class="rounded border border-white/20 px-2 py-1.5 opacity-85">
						<div class="flex items-start justify-between gap-2">
							<div class="min-w-0 break-words font-semibold">
								{observerName}
								<span class="ml-1 text-xs font-normal opacity-70">(observer)</span>
								{#if !info.connected}
									<span class="ml-1 text-xs font-normal opacity-70">({OFFLINE_STATUS_LABEL})</span>
								{/if}
							</div>
							<div class="flex flex-wrap justify-end gap-1.5">
								{#if isModerator || observerName === name}
									<button
										class="btn variant-filled shrink-0 px-2 py-0.5 text-xs"
										on:click={() => setObserver(observerName, false)}
									>
										{observerJoinActionLabel(info)}
									</button>
								{/if}
								{#if isModerator}
									<button
										class="btn variant-filled shrink-0 px-2 py-0.5 text-xs"
										on:click={() => kickPlayer(observerName)}
									>
										Kick
									</button>
								{/if}
							</div>
						</div>
					</div>
				{/each}
			</div>
			{#if stage !== 'Joining'}
				<div class="mt-3 rounded border border-white/20 px-2 py-2">
					<label class="flex items-start gap-3 text-sm">
						<input
							type="checkbox"
							class="mt-0.5 h-4 w-4 cursor-pointer accent-primary-500"
							checked={allowNewPlayersMidgame}
							on:change={updateAllowMidgameJoin}
						/>
						<span>Allow new players to join</span>
					</label>
				</div>
				<div class="mt-3 rounded border border-white/20 px-2 py-2">
					<p class="block text-sm font-semibold">Round controls</p>
					<div class="mt-2 space-y-2">
						<button
							class="btn variant-filled w-full"
							on:click={refreshHands}
							disabled={!isModerator || !canRefreshHands}
						>
							Refresh active player hands
						</button>
						{#if !canRefreshHands}
							<p class="text-xs opacity-70">{SETTINGS_EDIT_STAGE_HINT}</p>
						{/if}
					</div>
				</div>
				<div class="mt-3 rounded border border-white/20 px-2 py-2">
					<p class="block text-sm font-semibold">Storyteller win condition (W)</p>
					<p class="mt-1 text-xs opacity-75">
						Storyteller wins when at least W people are different from others (for example, right
						when others are wrong).
					</p>
					<label class="mt-2 flex items-start gap-3 text-sm">
						<input
							type="checkbox"
							class="mt-0.5 h-4 w-4 cursor-pointer accent-primary-500"
							checked={storytellerLossComplementAuto}
							on:change={updateStorytellerLossComplementAuto}
							disabled={!isModerator}
						/>
						<span>Auto-tune W from the number of actual guessers after voting</span>
					</label>
					<div class="mt-2 flex items-center gap-2">
						<input
							type="number"
							class="w-24 rounded border px-2 py-1 text-gray-700 shadow"
							min={storytellerWinConditionMin}
							max={storytellerWinConditionMax}
							step="1"
							value={storytellerWinCondition}
							on:change={updateStorytellerLossComplement}
							disabled={!isModerator || storytellerLossComplementAuto}
						/>
						<span class="text-xs opacity-75"
							>Range: {storytellerWinConditionMin}–{storytellerWinConditionMax}</span
						>
					</div>
				</div>
				<div class="mt-3 rounded border border-white/20 px-2 py-2">
					<p class="block text-sm font-semibold">Votes per guesser</p>
					<p class="mt-1 text-xs opacity-75">
						How many vote tokens each guesser can cast in voting.
					</p>
					<div class="mt-2 flex items-center gap-2">
						<input
							type="number"
							class="w-24 rounded border px-2 py-1 text-gray-700 shadow"
							min={votesPerGuesserMin}
							max={votesPerGuesserMax}
							step="1"
							value={votesPerGuesser}
							on:change={updateVotesPerGuesser}
							disabled={!isModerator || !canChangePreVotingSettings}
						/>
						<span class="text-xs opacity-75">Range: {votesPerGuesserMin}–{votesPerGuesserMax}</span>
					</div>
					{#if !canChangePreVotingSettings}
						<p class="mt-1 text-xs opacity-70">{SETTINGS_EDIT_STAGE_HINT}</p>
					{/if}
				</div>
				<div class="mt-3 rounded border border-white/20 px-2 py-2">
					<p class="block text-sm font-semibold">Stage timers</p>
					<p class="mt-1 text-xs opacity-75">
						Show shared countdowns for each live stage. Results stays untimed.
					</p>
					<div class="mt-3 space-y-3">
						<div class="rounded border border-white/15 px-2 py-2">
							<div class="flex items-start justify-between gap-3">
								<div>
									<p class="font-medium">Hint / storyteller choosing</p>
									<p class="text-xs opacity-70">Storyteller chooses a card and clue.</p>
								</div>
								<label class="flex items-center gap-2 text-sm">
									<input
										type="checkbox"
										class="h-4 w-4 cursor-pointer accent-primary-500"
										checked={hintChoosingTimerEnabled}
										on:change={updateHintChoosingTimerEnabled}
										disabled={!isModerator || !canChangePreVotingSettings}
									/>
									<span>Enabled</span>
								</label>
							</div>
							<div class="mt-2 flex items-center gap-2">
								<input
									type="number"
									class="w-24 rounded border px-2 py-1 text-gray-700 shadow"
									min={STAGE_TIMER_DURATION_MIN_S}
									max={STAGE_TIMER_DURATION_MAX_S}
									step="1"
									value={hintChoosingTimerDurationS}
									on:change={updateHintChoosingTimerDuration}
									disabled={!isModerator || !canChangePreVotingSettings}
								/>
								<span class="text-xs opacity-75"
									>{STAGE_TIMER_DURATION_MIN_S}–{STAGE_TIMER_DURATION_MAX_S}s</span
								>
							</div>
						</div>

						<div class="rounded border border-white/15 px-2 py-2">
							<div class="flex items-start justify-between gap-3">
								<div>
									<p class="font-medium">Card choosing</p>
									<p class="text-xs opacity-70">Guessers choose matching cards.</p>
								</div>
								<label class="flex items-center gap-2 text-sm">
									<input
										type="checkbox"
										class="h-4 w-4 cursor-pointer accent-primary-500"
										checked={cardChoosingTimerEnabled}
										on:change={updateCardChoosingTimerEnabled}
										disabled={!isModerator || !canChangePreVotingSettings}
									/>
									<span>Enabled</span>
								</label>
							</div>
							<div class="mt-2 flex items-center gap-2">
								<input
									type="number"
									class="w-24 rounded border px-2 py-1 text-gray-700 shadow"
									min={STAGE_TIMER_DURATION_MIN_S}
									max={STAGE_TIMER_DURATION_MAX_S}
									step="1"
									value={cardChoosingTimerDurationS}
									on:change={updateCardChoosingTimerDuration}
									disabled={!isModerator || !canChangePreVotingSettings}
								/>
								<span class="text-xs opacity-75"
									>{STAGE_TIMER_DURATION_MIN_S}–{STAGE_TIMER_DURATION_MAX_S}s</span
								>
							</div>
							<label class="mt-2 flex items-start gap-3 text-sm">
								<input
									type="checkbox"
									class="mt-0.5 h-4 w-4 cursor-pointer accent-primary-500"
									checked={forceCardChoosingTimer}
									on:change={updateForceCardChoosingTimer}
									disabled={!isModerator || !canChangePreVotingSettings}
								/>
								<span>Force timeout by auto-choosing random cards</span>
							</label>
						</div>

						<div class="rounded border border-white/15 px-2 py-2">
							<div class="flex items-start justify-between gap-3">
								<div>
									<p class="font-medium">Voting</p>
									<p class="text-xs opacity-70">Guessers vote on the storyteller's card.</p>
								</div>
								<label class="flex items-center gap-2 text-sm">
									<input
										type="checkbox"
										class="h-4 w-4 cursor-pointer accent-primary-500"
										checked={votingTimerEnabled}
										on:change={updateVotingTimerEnabled}
										disabled={!isModerator || !canChangePreVotingSettings}
									/>
									<span>Enabled</span>
								</label>
							</div>
							<div class="mt-2 flex items-center gap-2">
								<input
									type="number"
									class="w-24 rounded border px-2 py-1 text-gray-700 shadow"
									min={STAGE_TIMER_DURATION_MIN_S}
									max={STAGE_TIMER_DURATION_MAX_S}
									step="1"
									value={votingTimerDurationS}
									on:change={updateVotingTimerDuration}
									disabled={!isModerator || !canChangePreVotingSettings}
								/>
								<span class="text-xs opacity-75"
									>{STAGE_TIMER_DURATION_MIN_S}–{STAGE_TIMER_DURATION_MAX_S}s</span
								>
							</div>
							<label class="mt-2 flex items-start gap-3 text-sm">
								<input
									type="checkbox"
									class="mt-0.5 h-4 w-4 cursor-pointer accent-primary-500"
									checked={forceVotingTimer}
									on:change={updateForceVotingTimer}
									disabled={!isModerator || !canChangePreVotingSettings}
								/>
								<span>Force timeout by auto-submitting random votes</span>
							</label>
						</div>
					</div>
					{#if !canChangePreVotingSettings}
						<p class="mt-2 text-xs opacity-70">{SETTINGS_EDIT_STAGE_HINT}</p>
					{/if}
				</div>
				<div class="mt-3 rounded border border-white/20 px-2 py-2">
					<p class="block text-sm font-semibold">Random wrong-card disabling</p>
					<p class="mt-1 text-xs opacity-75">
						During voting, each player may privately have extra wrong cards greyed out and
						unvotable.
					</p>
					<div class="mt-2">
						<label class="block text-sm font-medium" for="voting-wrong-card-disable-preset">
							Preset
						</label>
						<select
							id="voting-wrong-card-disable-preset"
							class="mt-1 w-full rounded border px-3 py-2 text-gray-700 shadow"
							value={selectedVotingWrongCardDisablePresetId}
							on:change={updateVotingWrongCardDisablePreset}
							disabled={!isModerator || !canChangePreVotingSettings}
						>
							{#each VOTING_WRONG_CARD_DISABLE_PRESETS as preset}
								<option value={preset.id}>{preset.label}</option>
							{/each}
							<option value="custom">Custom</option>
						</select>
					</div>
					<details class="mt-3 rounded border border-white/20 px-3 py-2">
						<summary class="cursor-pointer text-sm font-semibold">Advanced editor</summary>
						<div class="mt-3 space-y-3">
							<div>
								<label class="block text-sm font-medium" for="voting-wrong-card-disable-max">
									Max X
								</label>
								<input
									id="voting-wrong-card-disable-max"
									type="number"
									min="0"
									max={MAX_VOTING_WRONG_CARD_DISABLE_X}
									step="1"
									class="mt-1 w-24 rounded border px-2 py-1 text-gray-700 shadow"
									value={votingWrongCardDisableMax}
									on:change={updateVotingWrongCardDisableMax}
									disabled={!isModerator || !canChangePreVotingSettings}
								/>
								<p class="mt-1 text-xs opacity-70">Range: 0–{MAX_VOTING_WRONG_CARD_DISABLE_X}</p>
							</div>
							<p class="text-xs opacity-70">
								Editing probabilities auto-normalizes the total to 100% and switches the setting to
								Custom.
							</p>
							{#each normalizedVotingWrongCardDisableDistribution as probability, index}
								<div class="rounded border border-white/15 px-2 py-2">
									<div class="flex items-center justify-between gap-2">
										<span class="text-sm font-semibold">X = {index}</span>
										<span class="text-xs opacity-75">{formatPercent(probability)}</span>
									</div>
									<div class="mt-2 flex items-center gap-2">
										<input
											type="range"
											min="0"
											max="100"
											step="1"
											class="h-2 flex-1 cursor-pointer accent-primary-500"
											value={Math.round(probability * 100)}
											on:change={(event) =>
												updateVotingWrongCardDisableProbabilityFromEvent(index, event)}
											disabled={!isModerator ||
												!canChangePreVotingSettings ||
												normalizedVotingWrongCardDisableDistribution.length === 1}
										/>
										<input
											type="number"
											min="0"
											max="100"
											step="1"
											class="w-20 rounded border px-2 py-1 text-gray-700 shadow"
											value={Math.round(probability * 100)}
											on:change={(event) =>
												updateVotingWrongCardDisableProbabilityFromEvent(index, event)}
											disabled={!isModerator ||
												!canChangePreVotingSettings ||
												normalizedVotingWrongCardDisableDistribution.length === 1}
										/>
										<span class="text-xs opacity-75">%</span>
									</div>
								</div>
							{/each}
						</div>
					</details>
					{#if !canChangePreVotingSettings}
						<p class="mt-1 text-xs opacity-70">{SETTINGS_EDIT_STAGE_HINT}</p>
					{/if}
				</div>
				<div class="mt-3 rounded border border-white/20 px-2 py-2">
					<p class="block text-sm font-semibold">Cards per hand</p>
					<div class="mt-2 flex items-center gap-2">
						<input
							type="number"
							class="w-24 rounded border px-2 py-1 text-gray-700 shadow"
							min={cardsPerHandMin}
							max={cardsPerHandMax}
							step="1"
							value={cardsPerHand}
							on:change={updateCardsPerHand}
							disabled={!isModerator || !canChangeCardsPerHand}
						/>
						<span class="text-xs opacity-75">Range: {cardsPerHandMin}–{cardsPerHandMax}</span>
					</div>
					{#if !canChangeCardsPerHand}
						<p class="mt-1 text-xs opacity-70">{SETTINGS_EDIT_STAGE_HINT}</p>
					{/if}
				</div>
				<div class="mt-3 rounded border border-white/20 px-2 py-2">
					<p class="block text-sm font-semibold">Nominations per guesser</p>
					<div class="mt-2 flex items-center gap-2">
						<input
							type="number"
							class="w-24 rounded border px-2 py-1 text-gray-700 shadow"
							min={nominationsPerGuesserMin}
							max={nominationsPerGuesserMax}
							step="1"
							value={nominationsPerGuesser}
							on:change={updateNominationsPerGuesser}
							disabled={!isModerator || !canChangePreVotingSettings}
						/>
						<span class="text-xs opacity-75"
							>Range: {nominationsPerGuesserMin}–{nominationsPerGuesserMax}</span
						>
					</div>
					{#if !canChangePreVotingSettings}
						<p class="mt-1 text-xs opacity-70">{SETTINGS_EDIT_STAGE_HINT}</p>
					{/if}
				</div>
				<div class="mt-3 rounded border border-white/20 px-2 py-2">
					<label class="flex items-start gap-3 text-sm">
						<input
							type="checkbox"
							class="mt-0.5 h-4 w-4 cursor-pointer accent-primary-500"
							checked={showVotingCardNumbers}
							on:change={updateShowVotingCardNumbers}
							disabled={!isModerator || !canChangePreVotingSettings}
						/>
						<span>Show card numbers in voting/results stages</span>
					</label>
					{#if !canChangePreVotingSettings}
						<p class="mt-1 text-xs opacity-70">{SETTINGS_EDIT_STAGE_HINT}</p>
					{/if}
				</div>
				<div class="mt-3 rounded border border-white/20 px-2 py-2">
					<p class="block text-sm font-semibold">Round-start random discards</p>
					<p class="mt-1 text-xs opacity-75">
						Discard N random cards from each active hand at round start, then top up.
					</p>
					<div class="mt-2 flex items-center gap-2">
						<input
							type="number"
							class="w-24 rounded border px-2 py-1 text-gray-700 shadow"
							min="0"
							max={roundStartDiscardCountMax}
							step="1"
							value={roundStartDiscardCount}
							on:change={updateRoundStartDiscardCount}
							disabled={!isModerator || !canChangePreVotingSettings}
						/>
						<span class="text-xs opacity-75">Range: 0–{roundStartDiscardCountMax}</span>
					</div>
					{#if !canChangePreVotingSettings}
						<p class="mt-1 text-xs opacity-70">{SETTINGS_EDIT_STAGE_HINT}</p>
					{/if}
				</div>
				<div class="mt-3 rounded border border-white/20 px-2 py-2">
					<label class="flex items-start gap-3 text-sm">
						<input
							type="checkbox"
							class="mt-0.5 h-4 w-4 cursor-pointer accent-primary-500"
							checked={bonusCorrectGuessOnThresholdCorrectLoss}
							on:change={updateBonusCorrectGuessOnThresholdCorrectLoss}
							disabled={!isModerator || !canChangePreVotingSettings}
						/>
						<span> Give +3 correct-guess base in threshold-correct storyteller-loss rounds </span>
					</label>
					{#if !canChangePreVotingSettings}
						<p class="mt-1 text-xs opacity-70">{SETTINGS_EDIT_STAGE_HINT}</p>
					{/if}
				</div>
				<div class="mt-3 rounded border border-white/20 px-2 py-2">
					<label class="flex items-start gap-3 text-sm">
						<input
							type="checkbox"
							class="mt-0.5 h-4 w-4 cursor-pointer accent-primary-500"
							checked={bonusDoubleVoteOnThresholdCorrectLoss}
							on:change={updateBonusDoubleVoteOnThresholdCorrectLoss}
							disabled={!isModerator || !canChangePreVotingSettings}
						/>
						<span> Give +1 double-vote bonus in threshold-correct storyteller-loss rounds </span>
					</label>
					{#if !canChangePreVotingSettings}
						<p class="mt-1 text-xs opacity-70">{SETTINGS_EDIT_STAGE_HINT}</p>
					{/if}
				</div>
			{/if}
		</details>
	{/if}
</div>
