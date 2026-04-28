<script lang="ts">
	import { browser } from '$app/environment';
	import MigrateDeviceButton from '$lib/MigrateDeviceButton.svelte';
	import type GameServer from '$lib/gameServer';
	import {
		beautyScoringMode as beautyScoringModeStore,
		beautyVotePointsDivisor as beautyVotePointsDivisorStore,
		beautyVotePointsDivisorEffective as beautyVotePointsDivisorEffectiveStore,
		beautyVotePointsDivisorMode as beautyVotePointsDivisorModeStore,
		beautyVotePointsDivisorPlayerCountBase as beautyVotePointsDivisorPlayerCountBaseStore
	} from '$lib/mostBeautiful';
	import {
		clueRatingEnabled as clueRatingEnabledStore,
		clueRatingMaxStars as clueRatingMaxStarsStore,
		clueRatingMaxStarsMax as clueRatingMaxStarsMaxStore,
		clueRatingMaxStarsMin as clueRatingMaxStarsMinStore,
		clueRatingTimerDurationS as clueRatingTimerDurationSStore,
		clueRatingTimerEnabled as clueRatingTimerEnabledStore,
		forceClueRatingTimer as forceClueRatingTimerStore
	} from '$lib/clueRating';
	import type {
		BeautyScoringMode,
		BeautyVotePointsDivisorMode,
		GameMode,
		ObserverInfo,
		PlayerInfo,
		StellaQueuedRevealMode
	} from '$lib/types';
	import { OFFLINE_STATUS_LABEL } from '$lib/presence';
	import { isStageChangeAudioSupported, unlockStageChangeAudio } from '$lib/stageChangeAudio';
	import {
		cardsFitToHeight,
		hideNonSelectedStellaRevealCards,
		stickyVotingCardNavigatorEnabled,
		stageChangeSoundCuesEnabled,
		stageChangeVisualCuesEnabled,
		transparentCardNameOverlays
	} from '$lib/viewOptions';
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

	const STORYTELLER_CHOOSING_ONLY_HINT = 'Can only be changed during storyteller choosing stage.';
	const BEFORE_VOTING_HINT = 'Can only be changed before voting starts.';
	const BEFORE_BEAUTY_VOTING_HINT = 'Can only be changed before beauty voting starts.';
	const BEFORE_CLUE_RATING_HINT = 'Can only be changed before clue rating starts.';
	const BEFORE_RESULTS_HINT = 'Can only be changed before results are revealed.';
	const LIVE_DIXIT_STAGE_HINT = 'Can only be changed during a live Dixit stage.';
	const STAGE_TIMER_DURATION_MIN_S = 1;
	const STAGE_TIMER_DURATION_MAX_S = 60 * 60;
	const STELLA_SCOUT_TIMER_DURATION_MIN_S = 0;
	const STELLA_WORD_PACK_UNSAVED_LABEL = 'Unsaved Wordpack';
	const BEAUTY_VOTE_POINTS_DIVISOR_MIN = 1.0;
	const BEAUTY_VOTE_POINTS_DIVISOR_MAX = 100.0;
	const BEAUTY_VOTE_POINTS_DIVISOR_STEP = 0.1;
	const BEAUTY_VOTE_POINTS_DIVISOR_PLAYER_COUNT_BASE_MIN = 1;
	const BEAUTY_VOTE_POINTS_DIVISOR_PLAYER_COUNT_BASE_MAX = 100;

	export let players: { [key: string]: PlayerInfo } = {};
	export let observers: { [key: string]: ObserverInfo } = {};
	export let name = '';
	export let creator = '';
	export let moderators: string[] = [];
	export let stage = '';
	export let activePlayer = '';
	export let gameMode: GameMode = 'dixit_plus';
	export let allowNewPlayersMidgame = true;
	export let storytellerLossComplement = 0;
	export let storytellerLossComplementMin = 0;
	export let storytellerLossComplementMax = 0;
	export let storytellerLossComplementAuto = true;
	export let storytellerPoolEnabled = false;
	export let storytellerPoolActive = false;
	export let storytellerPoolPlayers: string[] = [];
	export let storytellerSuccessPoints = 3;
	export let storytellerSuccessPointsMin = 0;
	export let storytellerSuccessPointsMax = 10;
	export let votesPerGuesser = 1;
	export let votesPerGuesserMin = 1;
	export let votesPerGuesserMax = 1;
	export let beautyEnabled = false;
	export let beautyVotesPerPlayer = 2;
	export let beautyVotesPerPlayerMin = 1;
	export let beautyVotesPerPlayerMax = 1;
	export let beautyAllowDuplicateVotes = false;
	export let beautySplitPointsOnTie = true;
	export let beautyPointsBonus = 2;
	export let beautyPointsBonusMin = 0;
	export let beautyPointsBonusMax = 10;
	export let beautyResultsDisplayMode: import('$lib/types').BeautyResultsDisplayMode = 'combined';
	export let showPreviousResultsDuringStorytellerChoosing = true;
	export let randomizeVotingCardOrderPerViewer = false;
	export let cardsPerHand = 12;
	export let cardsPerHandMin = 1;
	export let cardsPerHandMax = 18;
	export let nominationsPerGuesser = 1;
	export let nominationsPerGuesserMin = 1;
	export let nominationsPerGuesserMax = 1;
	export let bonusCorrectGuessOnThresholdCorrectLoss = true;
	export let doubleVoteBonusNormalPoints = 1;
	export let doubleVoteBonusTooManyWrongPoints = 1;
	export let doubleVoteBonusTooManyWrongFollowsNormal = true;
	export let doubleVoteBonusTooManyCorrectPoints = 1;
	export let doubleVoteBonusTooManyCorrectFollowsNormal = true;
	export let doubleVoteBonusPointsMin = 0;
	export let doubleVoteBonusPointsMax = 10;
	export let bonusThresholdLossTogglesApplyToAllStorytellerLossRounds = true;
	export let showVotingCardNumbers = true;
	export let roundStartDiscardAllUnpinned = true;
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
	$: isStellaMode = gameMode === 'stella';
	$: isDixitMode = gameMode !== 'stella';
	$: selfObserveBlocked =
		(stage === 'PlayersChoose' ||
			stage === 'Voting' ||
			stage === 'BeautyVoting' ||
			stage === 'ClueRating') &&
		activePlayer === name;
	$: canBecomeObserver =
		!isSelfObserver && stage !== 'Joining' && stage !== 'End' && !selfObserveBlocked;
	$: canChangeLiveDixitSettings =
		isDixitMode &&
		(stage === 'ActiveChooses' ||
			stage === 'PlayersChoose' ||
			stage === 'Voting' ||
			stage === 'BeautyVoting' ||
			stage === 'ClueRating' ||
			stage === 'Results' ||
			stage === 'BeautyResults');
	$: canChangeCardsPerHand = isDixitMode && stage === 'ActiveChooses';
	$: canChangeBeforeVotingSettings =
		isDixitMode && (stage === 'ActiveChooses' || stage === 'PlayersChoose');
	$: canChangeBeforeBeautyVotingSettings =
		isDixitMode && (stage === 'ActiveChooses' || stage === 'PlayersChoose' || stage === 'Voting');
	$: canChangeBeforeClueRatingSettings =
		isDixitMode &&
		(stage === 'ActiveChooses' ||
			stage === 'PlayersChoose' ||
			stage === 'Voting' ||
			stage === 'BeautyVoting');
	$: canChangeBeforeResultsSettings =
		isDixitMode &&
		(stage === 'ActiveChooses' ||
			stage === 'PlayersChoose' ||
			stage === 'Voting' ||
			stage === 'BeautyVoting');
	$: canRefreshHands = isDixitMode && stage === 'ActiveChooses';
	$: canChangeStageTimers =
		(isDixitMode && canChangeLiveDixitSettings) || (isStellaMode && stage === 'StellaAssociate');
	$: canChangeStellaSettings = isStellaMode && stage === 'StellaAssociate';
	$: canSwitchStellaWordPack =
		isStellaMode &&
		(stage === 'StellaAssociate' || stage === 'StellaReveal' || stage === 'StellaResults');
	$: canChangeVotingOrderRandomizationSetting = canChangeLiveDixitSettings;
	$: canChangeStorytellerScoringSettings = isDixitMode
		? canChangeLiveDixitSettings
		: stage !== 'End';
	$: canChangeNumberOverlaySetting = isStellaMode
		? stage === 'StellaAssociate'
		: canChangeLiveDixitSettings;
	$: votingOrderRandomizationHint = LIVE_DIXIT_STAGE_HINT;
	$: settingsEditStageHint = isStellaMode
		? 'Can only be changed during the Resonance associate stage.'
		: LIVE_DIXIT_STAGE_HINT;
	$: timerSettingsEditHint = isStellaMode
		? 'Can only be changed during the Resonance associate stage.'
		: LIVE_DIXIT_STAGE_HINT;
	$: storytellerWinCondition = storytellerLossComplement + 1;
	$: storytellerWinConditionMin = storytellerLossComplementMin + 1;
	$: storytellerWinConditionMax = storytellerLossComplementMax + 1;
	$: effectiveDoubleVoteBonusTooManyWrongPoints = doubleVoteBonusTooManyWrongFollowsNormal
		? doubleVoteBonusNormalPoints
		: doubleVoteBonusTooManyWrongPoints;
	$: effectiveDoubleVoteBonusTooManyCorrectPoints = doubleVoteBonusTooManyCorrectFollowsNormal
		? doubleVoteBonusNormalPoints
		: doubleVoteBonusTooManyCorrectPoints;
	$: storytellerPoolStatusLabel = !storytellerPoolEnabled
		? 'Disabled'
		: storytellerPoolActive
			? 'Active'
			: 'Inactive';
	$: storytellerPoolSummary =
		storytellerPoolPlayers.length > 0 ? storytellerPoolPlayers.join(', ') : 'No selected players.';
	$: roundStartDiscardCountMax = Math.max(0, cardsPerHand - 1);
	$: stellaWordPackSelectOptions = [
		...stellaWordPackPresetNames,
		...(stellaWordPackIsUnsaved ? [STELLA_WORD_PACK_UNSAVED_LABEL] : [])
	];
	$: selectedStellaWordPackKey = stellaWordPackIsUnsaved
		? STELLA_WORD_PACK_UNSAVED_LABEL
		: stellaSelectedWordPackName;
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
		stage === 'Voting' || stage === 'BeautyVoting' || stage === 'ClueRating'
			? selfJoinPending
				? 'Cancel pending join'
				: 'Join next round'
			: 'Join now';
	$: activePlayerPoints = Object.fromEntries(
		Object.entries(players).map(([playerName, info]) => [playerName, info.points])
	);
	$: formattedBeautyVotePointsDivisor = $beautyVotePointsDivisorStore.toFixed(1);
	$: formattedEffectiveBeautyVotePointsDivisor =
		$beautyVotePointsDivisorEffectiveStore === null
			? 'pending first beauty results'
			: $beautyVotePointsDivisorEffectiveStore.toFixed(1);
	const supportsStageChangeAudio = isStageChangeAudioSupported();

	function isPlayerModerator(playerName: string) {
		return moderatorSet.has(playerName);
	}

	function handleStageChangeSoundToggle() {
		if ($stageChangeSoundCuesEnabled) {
			void unlockStageChangeAudio();
		}
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

	function getActivePlayerScoreFloor(targetName: string, isObserverTarget: boolean) {
		const eligibleScores = Object.entries(activePlayerPoints)
			.filter(([playerName]) => isObserverTarget || playerName !== targetName)
			.map(([, points]) => points);
		if (eligibleScores.length === 0) return null;
		return Math.min(...eligibleScores);
	}

	function getRaiseScoreFloor(
		playerName: string,
		currentPoints: number | null,
		isObserverTarget: boolean
	) {
		const floor = getActivePlayerScoreFloor(playerName, isObserverTarget);
		if (floor === null) return null;
		const effectivePoints = currentPoints ?? 0;
		return floor > effectivePoints ? floor : null;
	}

	function raiseScoreToActiveMin(playerName: string) {
		if (!isModerator) return;
		gameServer.raiseScoreToActiveMin(playerName);
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
		if (stage !== 'Voting' && stage !== 'BeautyVoting' && stage !== 'ClueRating') return 'Join now';
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
		if (!isModerator || !canChangeStorytellerScoringSettings || storytellerLossComplementAuto) {
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
		if (!isModerator || !canChangeStorytellerScoringSettings) {
			input.checked = storytellerLossComplementAuto;
			return;
		}
		gameServer.setStorytellerLossComplementAuto(input.checked);
	}

	function updateStorytellerSuccessPoints(event: Event) {
		const input = event.currentTarget as HTMLInputElement;
		const parsed = Number(input.value);
		if (!isModerator || !canChangeStorytellerScoringSettings) {
			input.value = `${storytellerSuccessPoints}`;
			return;
		}
		if (
			!Number.isInteger(parsed) ||
			parsed < storytellerSuccessPointsMin ||
			parsed > storytellerSuccessPointsMax
		) {
			input.value = `${storytellerSuccessPoints}`;
			return;
		}
		if (parsed !== storytellerSuccessPoints) {
			gameServer.setStorytellerSuccessPoints(parsed);
		}
	}

	function updateDoubleVoteBonusNormalPoints(event: Event) {
		const input = event.currentTarget as HTMLInputElement;
		const parsed = Number(input.value);
		if (!isModerator || !canChangeStorytellerScoringSettings) {
			input.value = `${doubleVoteBonusNormalPoints}`;
			return;
		}
		if (
			!Number.isInteger(parsed) ||
			parsed < doubleVoteBonusPointsMin ||
			parsed > doubleVoteBonusPointsMax
		) {
			input.value = `${doubleVoteBonusNormalPoints}`;
			return;
		}
		if (parsed !== doubleVoteBonusNormalPoints) {
			gameServer.setDoubleVoteBonusNormalPoints(parsed);
		}
	}

	function updateDoubleVoteBonusTooManyWrongPoints(event: Event) {
		const input = event.currentTarget as HTMLInputElement;
		const parsed = Number(input.value);
		if (
			!isModerator ||
			!canChangeStorytellerScoringSettings ||
			doubleVoteBonusTooManyWrongFollowsNormal
		) {
			input.value = `${effectiveDoubleVoteBonusTooManyWrongPoints}`;
			return;
		}
		if (
			!Number.isInteger(parsed) ||
			parsed < doubleVoteBonusPointsMin ||
			parsed > doubleVoteBonusPointsMax
		) {
			input.value = `${effectiveDoubleVoteBonusTooManyWrongPoints}`;
			return;
		}
		if (parsed !== doubleVoteBonusTooManyWrongPoints) {
			gameServer.setDoubleVoteBonusTooManyWrongPoints(parsed);
		}
	}

	function updateDoubleVoteBonusTooManyWrongFollowsNormal(event: Event) {
		const input = event.currentTarget as HTMLInputElement;
		if (!isModerator || !canChangeStorytellerScoringSettings) {
			input.checked = doubleVoteBonusTooManyWrongFollowsNormal;
			return;
		}
		gameServer.setDoubleVoteBonusTooManyWrongFollowsNormal(input.checked);
	}

	function updateDoubleVoteBonusTooManyCorrectPoints(event: Event) {
		const input = event.currentTarget as HTMLInputElement;
		const parsed = Number(input.value);
		if (
			!isModerator ||
			!canChangeStorytellerScoringSettings ||
			doubleVoteBonusTooManyCorrectFollowsNormal
		) {
			input.value = `${effectiveDoubleVoteBonusTooManyCorrectPoints}`;
			return;
		}
		if (
			!Number.isInteger(parsed) ||
			parsed < doubleVoteBonusPointsMin ||
			parsed > doubleVoteBonusPointsMax
		) {
			input.value = `${effectiveDoubleVoteBonusTooManyCorrectPoints}`;
			return;
		}
		if (parsed !== doubleVoteBonusTooManyCorrectPoints) {
			gameServer.setDoubleVoteBonusTooManyCorrectPoints(parsed);
		}
	}

	function updateDoubleVoteBonusTooManyCorrectFollowsNormal(event: Event) {
		const input = event.currentTarget as HTMLInputElement;
		if (!isModerator || !canChangeStorytellerScoringSettings) {
			input.checked = doubleVoteBonusTooManyCorrectFollowsNormal;
			return;
		}
		gameServer.setDoubleVoteBonusTooManyCorrectFollowsNormal(input.checked);
	}

	function updateVotesPerGuesser(event: Event) {
		const input = event.currentTarget as HTMLInputElement;
		const parsed = Number(input.value);
		if (!isModerator || !canChangeBeforeVotingSettings) {
			input.value = `${votesPerGuesser}`;
			return;
		}
		if (!Number.isInteger(parsed) || parsed < votesPerGuesserMin || parsed > votesPerGuesserMax) {
			input.value = `${votesPerGuesser}`;
			return;
		}
		if (parsed !== votesPerGuesser) {
			gameServer.setVotesPerGuesser(parsed);
		}
	}

	function updateBeautyEnabled(event: Event) {
		const input = event.currentTarget as HTMLInputElement;
		if (!isModerator || !canChangeBeforeBeautyVotingSettings) {
			input.checked = beautyEnabled;
			return;
		}
		gameServer.setBeautyEnabled(input.checked);
	}

	function updateBeautyVotesPerPlayer(event: Event) {
		const input = event.currentTarget as HTMLInputElement;
		const parsed = Number(input.value);
		if (!isModerator || !canChangeBeforeBeautyVotingSettings) {
			input.value = `${beautyVotesPerPlayer}`;
			return;
		}
		if (
			!Number.isInteger(parsed) ||
			parsed < beautyVotesPerPlayerMin ||
			parsed > beautyVotesPerPlayerMax
		) {
			input.value = `${beautyVotesPerPlayer}`;
			return;
		}
		if (parsed !== beautyVotesPerPlayer) {
			gameServer.setBeautyVotesPerPlayer(parsed);
		}
	}

	function updateBeautyAllowDuplicateVotes(event: Event) {
		const input = event.currentTarget as HTMLInputElement;
		if (!isModerator || !canChangeBeforeBeautyVotingSettings) {
			input.checked = beautyAllowDuplicateVotes;
			return;
		}
		gameServer.setBeautyAllowDuplicateVotes(input.checked);
	}

	function updateBeautySplitPointsOnTie(event: Event) {
		const input = event.currentTarget as HTMLInputElement;
		if (!isModerator || !canChangeStorytellerScoringSettings) {
			input.checked = beautySplitPointsOnTie;
			return;
		}
		gameServer.setBeautySplitPointsOnTie(input.checked);
	}

	function updateBeautyPointsBonus(event: Event) {
		const input = event.currentTarget as HTMLInputElement;
		const parsed = Number(input.value);
		if (!isModerator || !canChangeStorytellerScoringSettings) {
			input.value = `${beautyPointsBonus}`;
			return;
		}
		if (
			!Number.isInteger(parsed) ||
			parsed < beautyPointsBonusMin ||
			parsed > beautyPointsBonusMax
		) {
			input.value = `${beautyPointsBonus}`;
			return;
		}
		if (parsed !== beautyPointsBonus) {
			gameServer.setBeautyPointsBonus(parsed);
		}
	}

	function updateBeautyScoringMode(event: Event) {
		const select = event.currentTarget as HTMLSelectElement;
		if (!isModerator || !canChangeStorytellerScoringSettings) {
			select.value = $beautyScoringModeStore;
			return;
		}
		gameServer.setBeautyScoringMode(select.value as BeautyScoringMode);
	}

	function updateBeautyVotePointsDivisor(event: Event) {
		const input = event.currentTarget as HTMLInputElement;
		const parsed = Number(input.value);
		if (!isModerator || !canChangeStorytellerScoringSettings) {
			input.value = formattedBeautyVotePointsDivisor;
			return;
		}
		if (
			!Number.isFinite(parsed) ||
			parsed < BEAUTY_VOTE_POINTS_DIVISOR_MIN ||
			parsed > BEAUTY_VOTE_POINTS_DIVISOR_MAX
		) {
			input.value = formattedBeautyVotePointsDivisor;
			return;
		}
		const normalized = Math.round(parsed * 10) / 10;
		if (normalized !== $beautyVotePointsDivisorStore) {
			gameServer.setBeautyVotePointsDivisor(normalized);
		}
	}

	function updateBeautyVotePointsDivisorMode(event: Event) {
		const select = event.currentTarget as HTMLSelectElement;
		if (!isModerator || !canChangeStorytellerScoringSettings) {
			select.value = $beautyVotePointsDivisorModeStore;
			return;
		}
		gameServer.setBeautyVotePointsDivisorMode(select.value as BeautyVotePointsDivisorMode);
	}

	function updateBeautyVotePointsDivisorPlayerCountBase(event: Event) {
		const input = event.currentTarget as HTMLInputElement;
		const parsed = Number(input.value);
		if (!isModerator || !canChangeStorytellerScoringSettings) {
			input.value = `${$beautyVotePointsDivisorPlayerCountBaseStore}`;
			return;
		}
		if (
			!Number.isInteger(parsed) ||
			parsed < BEAUTY_VOTE_POINTS_DIVISOR_PLAYER_COUNT_BASE_MIN ||
			parsed > BEAUTY_VOTE_POINTS_DIVISOR_PLAYER_COUNT_BASE_MAX
		) {
			input.value = `${$beautyVotePointsDivisorPlayerCountBaseStore}`;
			return;
		}
		if (parsed !== $beautyVotePointsDivisorPlayerCountBaseStore) {
			gameServer.setBeautyVotePointsDivisorPlayerCountBase(parsed);
		}
	}

	function updateClueRatingEnabled(event: Event) {
		const input = event.currentTarget as HTMLInputElement;
		if (!isModerator || !canChangeBeforeClueRatingSettings) {
			input.checked = $clueRatingEnabledStore;
			return;
		}
		gameServer.setClueRatingEnabled(input.checked);
	}

	function updateClueRatingMaxStars(event: Event) {
		const input = event.currentTarget as HTMLInputElement;
		const parsed = Number(input.value);
		if (!isModerator || !canChangeBeforeClueRatingSettings) {
			input.value = `${$clueRatingMaxStarsStore}`;
			return;
		}
		if (
			!Number.isInteger(parsed) ||
			parsed < $clueRatingMaxStarsMinStore ||
			parsed > $clueRatingMaxStarsMaxStore
		) {
			input.value = `${$clueRatingMaxStarsStore}`;
			return;
		}
		if (parsed !== $clueRatingMaxStarsStore) {
			gameServer.setClueRatingMaxStars(parsed);
		}
	}

	function updateShowPreviousResultsDuringStorytellerChoosing(event: Event) {
		const input = event.currentTarget as HTMLInputElement;
		if (!isModerator || !canChangeLiveDixitSettings) {
			input.checked = showPreviousResultsDuringStorytellerChoosing;
			return;
		}
		gameServer.setShowPreviousResultsDuringStorytellerChoosing(input.checked);
	}

	function updateCardsPerHand(event: Event) {
		const input = event.currentTarget as HTMLInputElement;
		const parsed = Number(input.value);
		if (!isModerator || !canChangeCardsPerHand) {
			input.value = `${cardsPerHand}`;
			return;
		}
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
		if (!isModerator || !canChangeBeforeVotingSettings) {
			input.value = `${nominationsPerGuesser}`;
			return;
		}
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
		if (!isModerator || !canChangeStorytellerScoringSettings) {
			input.checked = bonusCorrectGuessOnThresholdCorrectLoss;
			return;
		}
		gameServer.setBonusCorrectGuessOnThresholdCorrectLoss(input.checked);
	}

	function updateBonusThresholdLossTogglesApplyToAllStorytellerLossRounds(event: Event) {
		const input = event.currentTarget as HTMLInputElement;
		if (!isModerator || !canChangeStorytellerScoringSettings) {
			input.checked = bonusThresholdLossTogglesApplyToAllStorytellerLossRounds;
			return;
		}
		gameServer.setBonusThresholdLossTogglesApplyToAllStorytellerLossRounds(input.checked);
	}

	function updateShowVotingCardNumbers(event: Event) {
		const input = event.currentTarget as HTMLInputElement;
		if (!isModerator || !canChangeNumberOverlaySetting) {
			input.checked = showVotingCardNumbers;
			return;
		}
		gameServer.setShowVotingCardNumbers(input.checked);
	}

	function updateRandomizeVotingCardOrderPerViewer(event: Event) {
		const input = event.currentTarget as HTMLInputElement;
		if (!isModerator || !canChangeVotingOrderRandomizationSetting) {
			input.checked = randomizeVotingCardOrderPerViewer;
			return;
		}
		gameServer.setRandomizeVotingCardOrderPerViewer(input.checked);
	}

	function updateRoundStartDiscardCount(event: Event) {
		const input = event.currentTarget as HTMLInputElement;
		const parsed = Number(input.value);
		if (!isModerator || !canChangeLiveDixitSettings || roundStartDiscardAllUnpinned) {
			input.value = `${roundStartDiscardCount}`;
			return;
		}
		if (!Number.isInteger(parsed) || parsed < 0 || parsed > roundStartDiscardCountMax) {
			input.value = `${roundStartDiscardCount}`;
			return;
		}
		if (parsed !== roundStartDiscardCount) {
			gameServer.setRoundStartDiscardCount(parsed);
		}
	}

	function updateRoundStartDiscardAllUnpinned(event: Event) {
		const input = event.currentTarget as HTMLInputElement;
		if (!isModerator || !canChangeLiveDixitSettings) {
			input.checked = roundStartDiscardAllUnpinned;
			return;
		}
		gameServer.setRoundStartDiscardAllUnpinned(input.checked);
	}

	function updateStageTimerToggle(
		event: Event,
		currentValue: boolean,
		setter: (enabled: boolean) => void
	) {
		const input = event.currentTarget as HTMLInputElement;
		if (!isModerator || !canChangeStageTimers) {
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
		if (!isModerator || !canChangeStageTimers) {
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

	function updateForceHintChoosingTimer(event: Event) {
		updateStageTimerToggle(event, forceHintChoosingTimer, (enabled) =>
			gameServer.setForceHintChoosingTimer(enabled)
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

	function updateBeautyTimerEnabled(event: Event) {
		updateStageTimerToggle(event, beautyTimerEnabled, (enabled) =>
			gameServer.setBeautyTimerEnabled(enabled)
		);
	}

	function updateBeautyTimerDuration(event: Event) {
		updateStageTimerDuration(event, beautyTimerDurationS, (seconds) =>
			gameServer.setBeautyTimerDuration(seconds)
		);
	}

	function updateClueRatingTimerEnabled(event: Event) {
		updateStageTimerToggle(event, $clueRatingTimerEnabledStore, (enabled) =>
			gameServer.setClueRatingTimerEnabled(enabled)
		);
	}

	function updateClueRatingTimerDuration(event: Event) {
		updateStageTimerDuration(event, $clueRatingTimerDurationSStore, (seconds) =>
			gameServer.setClueRatingTimerDuration(seconds)
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

	function updateForceBeautyTimer(event: Event) {
		updateStageTimerToggle(event, forceBeautyTimer, (enabled) =>
			gameServer.setForceBeautyTimer(enabled)
		);
	}

	function updateForceClueRatingTimer(event: Event) {
		updateStageTimerToggle(event, $forceClueRatingTimerStore, (enabled) =>
			gameServer.setForceClueRatingTimer(enabled)
		);
	}

	function updateStellaBoardSize(event: Event) {
		const input = event.currentTarget as HTMLInputElement;
		const parsed = Number(input.value);
		if (!isModerator || !canChangeStellaSettings) {
			input.value = `${stellaBoardSize}`;
			return;
		}
		if (!Number.isInteger(parsed) || parsed < stellaBoardSizeMin || parsed > stellaBoardSizeMax) {
			input.value = `${stellaBoardSize}`;
			return;
		}
		if (parsed !== stellaBoardSize) {
			gameServer.setStellaBoardSize(parsed);
		}
	}

	function updateStellaSelectionMin(event: Event) {
		const input = event.currentTarget as HTMLInputElement;
		const parsed = Number(input.value);
		if (!isModerator || !canChangeStellaSettings) {
			input.value = `${stellaSelectionMin}`;
			return;
		}
		if (
			!Number.isInteger(parsed) ||
			parsed < stellaSelectionCountMin ||
			parsed > stellaSelectionCountMax
		) {
			input.value = `${stellaSelectionMin}`;
			return;
		}
		if (parsed !== stellaSelectionMin) {
			gameServer.setStellaSelectionMin(parsed);
		}
	}

	function updateStellaSelectionMax(event: Event) {
		const input = event.currentTarget as HTMLInputElement;
		const parsed = Number(input.value);
		if (!isModerator || !canChangeStellaSettings) {
			input.value = `${stellaSelectionMax}`;
			return;
		}
		if (
			!Number.isInteger(parsed) ||
			parsed < stellaSelectionCountMin ||
			parsed > stellaSelectionCountMax
		) {
			input.value = `${stellaSelectionMax}`;
			return;
		}
		if (parsed !== stellaSelectionMax) {
			gameServer.setStellaSelectionMax(parsed);
		}
	}

	function updateStellaQueueDuringAssociation(event: Event) {
		const input = event.currentTarget as HTMLInputElement;
		if (!isModerator || !canChangeStellaSettings) {
			input.checked = stellaQueueDuringAssociation;
			return;
		}
		gameServer.setStellaQueueDuringAssociation(input.checked);
	}

	function updateStellaQueuedRevealMode(event: Event) {
		const select = event.currentTarget as HTMLSelectElement;
		if (!isModerator || !canChangeStellaSettings) {
			select.value = stellaQueuedRevealMode;
			return;
		}
		if (select.value !== stellaQueuedRevealMode) {
			gameServer.setStellaQueuedRevealMode(select.value as StellaQueuedRevealMode);
		}
	}

	function updateStellaScoutTimerEnabled(event: Event) {
		const input = event.currentTarget as HTMLInputElement;
		if (!isModerator || !canChangeStellaSettings) {
			input.checked = stellaScoutTimerEnabled;
			return;
		}
		gameServer.setStellaScoutTimerEnabled(input.checked);
	}

	function updateStellaScoutTimerDuration(event: Event) {
		const input = event.currentTarget as HTMLInputElement;
		const parsed = Number(input.value);
		if (!isModerator || !canChangeStellaSettings) {
			input.value = `${stellaScoutTimerDurationS}`;
			return;
		}
		if (
			!Number.isInteger(parsed) ||
			parsed < STELLA_SCOUT_TIMER_DURATION_MIN_S ||
			parsed > STAGE_TIMER_DURATION_MAX_S
		) {
			input.value = `${stellaScoutTimerDurationS}`;
			return;
		}
		if (parsed !== stellaScoutTimerDurationS) {
			gameServer.setStellaScoutTimerDuration(parsed);
		}
	}

	function updateForceStellaScoutTimer(event: Event) {
		const input = event.currentTarget as HTMLInputElement;
		if (!isModerator || !canChangeStellaSettings) {
			input.checked = forceStellaScoutTimer;
			return;
		}
		gameServer.setForceStellaScoutTimer(input.checked);
	}

	function updateStellaWordPack(event: Event) {
		const select = event.currentTarget as HTMLSelectElement;
		if (!isModerator || !canSwitchStellaWordPack) {
			select.value = selectedStellaWordPackKey;
			return;
		}
		if (
			!select.value ||
			select.value === selectedStellaWordPackKey ||
			select.value === STELLA_WORD_PACK_UNSAVED_LABEL
		) {
			select.value = selectedStellaWordPackKey;
			return;
		}
		gameServer.setStellaWordPackPreset(select.value);
	}

	function formatPercent(probability: number) {
		return `${(probability * 100).toFixed(1).replace(/\\.0$/, '')}%`;
	}

	function updateVotingWrongCardDisablePreset(event: Event) {
		const select = event.currentTarget as HTMLSelectElement;
		if (!isModerator || !canChangeBeforeVotingSettings) {
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
		if (!isModerator || !canChangeBeforeVotingSettings) {
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
		if (!isModerator || !canChangeBeforeVotingSettings) return;

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
	<div class="space-y-2">
		<MigrateDeviceButton fullWidth={true} />
		<p class="text-xs opacity-70">
			Copy a room-specific link that moves this room identity, plus the room password when set, to
			another device.
		</p>
	</div>
	<label class="flex items-start gap-3">
		<input
			type="checkbox"
			class="mt-1 h-4 w-4 cursor-pointer accent-primary-500"
			bind:checked={$cardsFitToHeight}
		/>
		<span class="block font-medium">Cards fit to height</span>
	</label>
	<label class="flex items-start gap-3">
		<input
			type="checkbox"
			class="mt-1 h-4 w-4 cursor-pointer accent-primary-500 disabled:cursor-not-allowed disabled:opacity-50"
			bind:checked={$stageChangeSoundCuesEnabled}
			disabled={!supportsStageChangeAudio}
			on:change={handleStageChangeSoundToggle}
		/>
		<div>
			<span class="block font-medium">Stage change sound cues</span>
			<p class="text-xs opacity-70">
				Short tones for stage changes, including pause/game-over, plus Resonance scout turns.
			</p>
			{#if !supportsStageChangeAudio}
				<p class="mt-1 text-xs opacity-70">Not supported in this browser.</p>
			{/if}
		</div>
	</label>
	<label class="flex items-start gap-3">
		<input
			type="checkbox"
			class="mt-1 h-4 w-4 cursor-pointer accent-primary-500"
			bind:checked={$stageChangeVisualCuesEnabled}
		/>
		<div>
			<span class="block font-medium">Stage change visual cues</span>
			<p class="text-xs opacity-70">
				Brief screen-edge flash when the room enters a new stage. Local view option only.
			</p>
		</div>
	</label>
	<label class="flex items-start gap-3">
		<input
			type="checkbox"
			class="mt-1 h-4 w-4 cursor-pointer accent-primary-500"
			bind:checked={$transparentCardNameOverlays}
		/>
		<div>
			<span class="block font-medium">Transparent card name overlays</span>
			<p class="text-xs opacity-70">
				Local view option for Talespin results and Resonance revealed-card chooser labels.
			</p>
		</div>
	</label>
	{#if isDixitMode && stage !== 'Joining' && stage !== 'End'}
		<label class="flex items-start gap-3">
			<input
				type="checkbox"
				class="mt-1 h-4 w-4 cursor-pointer accent-primary-500"
				bind:checked={$stickyVotingCardNavigatorEnabled}
			/>
			<div>
				<span class="block font-medium">Sticky card navigator</span>
				<p class="text-xs opacity-70">
					Local view option for Dixit voting and results. Shows a compact sticky grid of canonical
					card numbers for quick jumping, even if card badges are hidden.
				</p>
			</div>
		</label>
	{/if}
	{#if isStellaMode && stage !== 'Joining' && stage !== 'End'}
		<label class="flex items-start gap-3">
			<input
				type="checkbox"
				class="mt-1 h-4 w-4 cursor-pointer accent-primary-500"
				bind:checked={$hideNonSelectedStellaRevealCards}
			/>
			<div>
				<span class="block font-medium">Hide non-selected cards during Resonance reveal</span>
				<p class="text-xs opacity-70">
					Local view option for your reveal turns only; it only affects your device.
				</p>
			</div>
		</label>
	{/if}
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
								{#if isModerator && getRaiseScoreFloor(playerName, players[playerName]?.points ?? 0, false) !== null}
									<button
										class="btn variant-filled shrink-0 px-2 py-0.5 text-xs"
										title="Raise score to active-player floor"
										aria-label="Raise score to active-player floor"
										on:click={() => raiseScoreToActiveMin(playerName)}
									>
										⏫
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
								{#if isModerator && getRaiseScoreFloor(observerName, info.points, true) !== null}
									<button
										class="btn variant-filled shrink-0 px-2 py-0.5 text-xs"
										title="Raise score to active-player floor"
										aria-label="Raise score to active-player floor"
										on:click={() => raiseScoreToActiveMin(observerName)}
									>
										⏫
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
					<p class="block text-sm font-semibold">Stage timers</p>
					<p class="mt-1 text-xs opacity-75">
						{isStellaMode
							? 'Associate uses the shared Resonance countdown. Queued/manual reveal pacing is configured below.'
							: 'Show shared countdowns for each live stage. Results stays untimed.'}
					</p>
					<div class="mt-3 space-y-3">
						<div class="rounded border border-white/15 px-2 py-2">
							<div class="flex items-start justify-between gap-3">
								<div>
									<p class="font-medium">
										{isStellaMode ? 'Associate' : 'Hint / storyteller choosing'}
									</p>
									<p class="text-xs opacity-70">
										{isStellaMode
											? 'Players pick every board card that matches the clue.'
											: 'Storyteller chooses a card and clue.'}
									</p>
								</div>
								<label class="flex items-center gap-2 text-sm">
									<input
										type="checkbox"
										class="h-4 w-4 cursor-pointer accent-primary-500"
										checked={hintChoosingTimerEnabled}
										on:change={updateHintChoosingTimerEnabled}
										disabled={!isModerator || !canChangeStageTimers}
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
									disabled={!isModerator || !canChangeStageTimers}
								/>
								<span class="text-xs opacity-75"
									>{STAGE_TIMER_DURATION_MIN_S}–{STAGE_TIMER_DURATION_MAX_S}s</span
								>
							</div>
							<label class="mt-2 flex items-start gap-3 text-sm">
								<input
									type="checkbox"
									class="mt-0.5 h-4 w-4 cursor-pointer accent-primary-500"
									checked={forceHintChoosingTimer}
									on:change={updateForceHintChoosingTimer}
									disabled={!isModerator || !canChangeStageTimers}
								/>
								<span>
									{isStellaMode
										? 'Force timeout by locking random selections from the median manual count'
										: 'Force timeout by switching to a different storyteller'}
								</span>
							</label>
						</div>

						{#if isDixitMode}
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
											disabled={!isModerator || !canChangeStageTimers}
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
										disabled={!isModerator || !canChangeStageTimers}
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
										disabled={!isModerator || !canChangeStageTimers}
									/>
									<span>Force timeout by auto-choosing random cards</span>
								</label>
							</div>
						{/if}

						{#if isDixitMode}
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
											disabled={!isModerator || !canChangeStageTimers}
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
										disabled={!isModerator || !canChangeStageTimers}
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
										disabled={!isModerator || !canChangeStageTimers}
									/>
									<span>Force timeout by auto-submitting random votes</span>
								</label>
							</div>
							{#if isDixitMode}
								<div class="rounded border border-white/15 px-2 py-2">
									<div class="flex items-start justify-between gap-3">
										<div>
											<p class="font-medium">Most Beautiful</p>
											<p class="text-xs opacity-70">Optional beauty voting countdown.</p>
										</div>
										<label class="flex items-center gap-2 text-sm">
											<input
												type="checkbox"
												class="h-4 w-4 cursor-pointer accent-primary-500"
												checked={beautyTimerEnabled}
												on:change={updateBeautyTimerEnabled}
												disabled={!isModerator || !canChangeStageTimers}
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
											value={beautyTimerDurationS}
											on:change={updateBeautyTimerDuration}
											disabled={!isModerator || !canChangeStageTimers}
										/>
										<span class="text-xs opacity-75"
											>{STAGE_TIMER_DURATION_MIN_S}–{STAGE_TIMER_DURATION_MAX_S}s</span
										>
									</div>
									<label class="mt-2 flex items-start gap-3 text-sm">
										<input
											type="checkbox"
											class="mt-0.5 h-4 w-4 cursor-pointer accent-primary-500"
											checked={forceBeautyTimer}
											on:change={updateForceBeautyTimer}
											disabled={!isModerator || !canChangeStageTimers}
										/>
										<span>Force timeout by skipping missing beauty votes</span>
									</label>
								</div>
								<div class="rounded border border-white/15 px-2 py-2">
									<div class="flex items-start justify-between gap-3">
										<div>
											<p class="font-medium">Clue rating</p>
											<p class="text-xs opacity-70">Optional star-voting countdown.</p>
										</div>
										<label class="flex items-center gap-2 text-sm">
											<input
												type="checkbox"
												class="h-4 w-4 cursor-pointer accent-primary-500"
												checked={$clueRatingTimerEnabledStore}
												on:change={updateClueRatingTimerEnabled}
												disabled={!isModerator || !canChangeStageTimers}
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
											value={$clueRatingTimerDurationSStore}
											on:change={updateClueRatingTimerDuration}
											disabled={!isModerator || !canChangeStageTimers}
										/>
										<span class="text-xs opacity-75"
											>{STAGE_TIMER_DURATION_MIN_S}–{STAGE_TIMER_DURATION_MAX_S}s</span
										>
									</div>
									<label class="mt-2 flex items-start gap-3 text-sm">
										<input
											type="checkbox"
											class="mt-0.5 h-4 w-4 cursor-pointer accent-primary-500"
											checked={$forceClueRatingTimerStore}
											on:change={updateForceClueRatingTimer}
											disabled={!isModerator || !canChangeStageTimers}
										/>
										<span>Force timeout by skipping missing star votes</span>
									</label>
								</div>
							{/if}
						{/if}
					</div>
					{#if !canChangeStageTimers}
						<p class="mt-2 text-xs opacity-70">{timerSettingsEditHint}</p>
					{/if}
				</div>
				{#if isStellaMode}
					<div class="mt-3 rounded border border-white/20 px-2 py-2">
						<p class="block text-sm font-semibold">Resonance settings</p>
						<p class="mt-1 text-xs opacity-75">
							Live Resonance settings for the current board and the next clue reset.
						</p>
						<div class="mt-3 space-y-3">
							<div>
								<label class="block text-sm font-medium" for="sidebar-stella-board-size">
									Board size
								</label>
								<div class="mt-2 flex items-center gap-2">
									<input
										id="sidebar-stella-board-size"
										type="number"
										class="w-24 rounded border px-2 py-1 text-gray-700 shadow"
										min={stellaBoardSizeMin}
										max={stellaBoardSizeMax}
										step="1"
										value={stellaBoardSize}
										on:change={updateStellaBoardSize}
										disabled={!isModerator || !canChangeStellaSettings}
									/>
									<span class="text-xs opacity-75">
										Range: {stellaBoardSizeMin}–{stellaBoardSizeMax}
									</span>
								</div>
							</div>
							<div class="grid grid-cols-2 gap-3">
								<div>
									<label class="block text-sm font-medium" for="sidebar-stella-selection-min">
										Selection min
									</label>
									<input
										id="sidebar-stella-selection-min"
										type="number"
										class="mt-2 w-full rounded border px-2 py-1 text-gray-700 shadow"
										min={stellaSelectionCountMin}
										max={stellaSelectionCountMax}
										step="1"
										value={stellaSelectionMin}
										on:change={updateStellaSelectionMin}
										disabled={!isModerator || !canChangeStellaSettings}
									/>
								</div>
								<div>
									<label class="block text-sm font-medium" for="sidebar-stella-selection-max">
										Selection max
									</label>
									<input
										id="sidebar-stella-selection-max"
										type="number"
										class="mt-2 w-full rounded border px-2 py-1 text-gray-700 shadow"
										min={stellaSelectionCountMin}
										max={stellaSelectionCountMax}
										step="1"
										value={stellaSelectionMax}
										on:change={updateStellaSelectionMax}
										disabled={!isModerator || !canChangeStellaSettings}
									/>
								</div>
							</div>
							<p class="text-xs opacity-75">
								Selection range: {stellaSelectionCountMin}–{stellaSelectionCountMax}
							</p>
							<div class="rounded border border-white/15 px-2 py-2 space-y-3">
								<label class="flex items-start gap-3 text-sm">
									<input
										type="checkbox"
										class="mt-0.5 h-4 w-4 cursor-pointer accent-primary-500"
										checked={stellaQueueDuringAssociation}
										on:change={updateStellaQueueDuringAssociation}
										disabled={!isModerator || !canChangeStellaSettings}
									/>
									<div>
										<span class="block font-medium">Queue during Association</span>
										<p class="text-xs opacity-70">
											Players order reveals while selecting cards. When enabled, there is no manual
											scout click-to-reveal step.
										</p>
									</div>
								</label>
								<div>
									<label class="block text-sm font-medium" for="sidebar-stella-reveal-mode">
										Queued reveal playback
									</label>
									<select
										id="sidebar-stella-reveal-mode"
										class="mt-2 w-full rounded border px-2 py-1 text-gray-700 shadow"
										value={stellaQueuedRevealMode}
										on:change={updateStellaQueuedRevealMode}
										disabled={!isModerator ||
											!canChangeStellaSettings ||
											!stellaQueueDuringAssociation}
									>
										<option value="animated">Animated reveal</option>
										<option value="fast">Fast reveal</option>
									</select>
								</div>
							</div>
							<div class="rounded border border-white/15 px-2 py-2 space-y-3">
								<p class="font-medium">Manual reveal fallback</p>
								<p class="text-xs opacity-70">
									These only apply when Queue during Association is turned off.
								</p>
								<label class="flex items-center gap-2 text-sm">
									<input
										type="checkbox"
										class="h-4 w-4 cursor-pointer accent-primary-500"
										checked={stellaScoutTimerEnabled}
										on:change={updateStellaScoutTimerEnabled}
										disabled={!isModerator || !canChangeStellaSettings}
									/>
									<span>Scout timer enabled</span>
								</label>
								<div class="flex items-center gap-2">
									<input
										type="number"
										class="w-24 rounded border px-2 py-1 text-gray-700 shadow"
										min={STELLA_SCOUT_TIMER_DURATION_MIN_S}
										max={STAGE_TIMER_DURATION_MAX_S}
										step="1"
										value={stellaScoutTimerDurationS}
										on:change={updateStellaScoutTimerDuration}
										disabled={!isModerator || !canChangeStellaSettings}
									/>
									<span class="text-xs opacity-75">
										{STELLA_SCOUT_TIMER_DURATION_MIN_S}–{STAGE_TIMER_DURATION_MAX_S}s
									</span>
								</div>
								<label class="flex items-start gap-3 text-sm">
									<input
										type="checkbox"
										class="mt-0.5 h-4 w-4 cursor-pointer accent-primary-500"
										checked={forceStellaScoutTimer}
										on:change={updateForceStellaScoutTimer}
										disabled={!isModerator || !canChangeStellaSettings}
									/>
									<span>Force timeout by auto-revealing a random queued card</span>
								</label>
							</div>
							{#if canSwitchStellaWordPack}
								<div>
									<label class="block text-sm font-medium" for="sidebar-stella-word-pack">
										Word pack
									</label>
									<p class="mt-1 text-xs opacity-75">
										Switches the pack for the next clue reset or next Resonance round.
									</p>
									<div class="mt-2 flex items-center gap-2">
										<select
											id="sidebar-stella-word-pack"
											class="min-w-0 flex-1 rounded border px-2 py-1 text-gray-700 shadow"
											value={selectedStellaWordPackKey}
											on:change={updateStellaWordPack}
											disabled={!isModerator}
										>
											<option value="" disabled={!selectedStellaWordPackKey}>
												Choose a word pack preset
											</option>
											{#each stellaWordPackSelectOptions as optionName}
												<option
													value={optionName}
													disabled={optionName === STELLA_WORD_PACK_UNSAVED_LABEL}
												>
													{optionName}
												</option>
											{/each}
										</select>
									</div>
								</div>
							{/if}
						</div>
						{#if !isModerator}
							<p class="mt-2 text-xs opacity-70">
								Only moderators can edit live Resonance settings.
							</p>
						{:else if !canChangeStellaSettings && !canSwitchStellaWordPack}
							<p class="mt-2 text-xs opacity-70">{settingsEditStageHint}</p>
						{/if}
					</div>
				{/if}
				<div class="mt-3 rounded border border-white/20 px-2 py-2">
					<label class="flex items-start gap-3 text-sm">
						<input
							type="checkbox"
							class="mt-0.5 h-4 w-4 cursor-pointer accent-primary-500"
							checked={showVotingCardNumbers}
							on:change={updateShowVotingCardNumbers}
							disabled={!isModerator || !canChangeNumberOverlaySetting}
						/>
						<span>
							{isStellaMode
								? 'Show number overlays in all Resonance board stages'
								: 'Show card numbers in voting/results stages'}
						</span>
					</label>
					{#if !canChangeNumberOverlaySetting}
						<p class="mt-1 text-xs opacity-70">{settingsEditStageHint}</p>
					{/if}
				</div>
				{#if isDixitMode}
					<div class="mt-3 rounded border border-white/20 px-2 py-2">
						<label class="flex items-start gap-3 text-sm">
							<input
								type="checkbox"
								class="mt-0.5 h-4 w-4 cursor-pointer accent-primary-500"
								checked={randomizeVotingCardOrderPerViewer}
								on:change={updateRandomizeVotingCardOrderPerViewer}
								disabled={!isModerator || !canChangeVotingOrderRandomizationSetting}
							/>
							<div>
								<span class="block font-medium"> Randomize voting card order per player </span>
								<p class="text-xs opacity-75">
									Voting and Most Beautiful show each viewer a stable per-round shuffle, while the
									number badges keep the original canonical order. Results stay canonical.
								</p>
							</div>
						</label>
						{#if !canChangeVotingOrderRandomizationSetting}
							<p class="mt-1 text-xs opacity-70">{votingOrderRandomizationHint}</p>
						{/if}
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
								<p class="text-xs opacity-70">{STORYTELLER_CHOOSING_ONLY_HINT}</p>
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
								disabled={!isModerator || !canChangeStorytellerScoringSettings}
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
								disabled={!isModerator ||
									!canChangeStorytellerScoringSettings ||
									storytellerLossComplementAuto}
							/>
							<span class="text-xs opacity-75">
								Range: {storytellerWinConditionMin}–{storytellerWinConditionMax}
							</span>
						</div>
						{#if !canChangeStorytellerScoringSettings}
							<p class="mt-1 text-xs opacity-70">{LIVE_DIXIT_STAGE_HINT}</p>
						{/if}
					</div>
					<div class="mt-3 rounded border border-white/20 px-2 py-2">
						<p class="block text-sm font-semibold">Storyteller success score</p>
						<p class="mt-1 text-xs opacity-75">
							Points awarded to the storyteller in a normal successful round.
						</p>
						<div class="mt-2 flex items-center gap-2">
							<input
								type="number"
								class="w-24 rounded border px-2 py-1 text-gray-700 shadow"
								min={storytellerSuccessPointsMin}
								max={storytellerSuccessPointsMax}
								step="1"
								value={storytellerSuccessPoints}
								on:change={updateStorytellerSuccessPoints}
								disabled={!isModerator || !canChangeStorytellerScoringSettings}
							/>
							<span class="text-xs opacity-75">
								Range: {storytellerSuccessPointsMin}–{storytellerSuccessPointsMax}
							</span>
						</div>
						{#if !canChangeStorytellerScoringSettings}
							<p class="mt-1 text-xs opacity-70">{LIVE_DIXIT_STAGE_HINT}</p>
						{/if}
					</div>
					<div class="mt-3 rounded border border-white/20 px-2 py-2">
						<div class="flex items-start justify-between gap-3">
							<div>
								<p class="block text-sm font-semibold">Storyteller pool</p>
								<p class="mt-1 text-xs opacity-75">
									Lobby-only moderator option that restricts storyteller selection to a saved subset
									of players.
								</p>
							</div>
							<span class="rounded border border-white/20 px-2 py-0.5 text-xs opacity-80">
								{storytellerPoolStatusLabel}
							</span>
						</div>
						<p class="mt-2 text-xs opacity-75">
							{#if storytellerPoolEnabled && !storytellerPoolActive}
								Falls back to unrestricted storyteller selection until one of the selected players
								is active again.
							{:else if !storytellerPoolEnabled}
								Restricted storyteller selection is off.
							{:else}
								Only the selected active players can be chosen as storyteller.
							{/if}
						</p>
						<p class="mt-2 text-xs opacity-75">
							Selected: {storytellerPoolSummary}
						</p>
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
								disabled={!isModerator || !canChangeBeforeVotingSettings}
							/>
							<span class="text-xs opacity-75"
								>Range: {votesPerGuesserMin}–{votesPerGuesserMax}</span
							>
						</div>
						{#if !canChangeBeforeVotingSettings}
							<p class="mt-1 text-xs opacity-70">{BEFORE_VOTING_HINT}</p>
						{/if}
					</div>
					<div class="mt-3 rounded border border-white/20 px-2 py-2">
						<p class="block text-sm font-semibold">Most Beautiful</p>
						<p class="mt-1 text-xs opacity-75">
							Optional extra vote after storyteller voting and before results are revealed.
						</p>
						<label class="mt-2 flex items-start gap-3 text-sm">
							<input
								type="checkbox"
								class="mt-0.5 h-4 w-4 cursor-pointer accent-primary-500"
								checked={beautyEnabled}
								on:change={updateBeautyEnabled}
								disabled={!isModerator || !canChangeBeforeBeautyVotingSettings}
							/>
							<span>Enable Most Beautiful stage</span>
						</label>
						<div class="mt-3 grid gap-3 md:grid-cols-2">
							<div>
								<label class="text-sm font-medium" for="beauty-votes-per-player">
									Beauty votes per player
								</label>
								<input
									id="beauty-votes-per-player"
									type="number"
									class="mt-1 w-full rounded border px-2 py-1 text-gray-700 shadow"
									min={beautyVotesPerPlayerMin}
									max={beautyVotesPerPlayerMax}
									step="1"
									value={beautyVotesPerPlayer}
									on:change={updateBeautyVotesPerPlayer}
									disabled={!isModerator || !canChangeBeforeBeautyVotingSettings}
								/>
								<p class="mt-1 text-xs opacity-75">
									Range: {beautyVotesPerPlayerMin}–{beautyVotesPerPlayerMax}
								</p>
							</div>
							<div>
								<label class="text-sm font-medium" for="beauty-scoring-mode">
									Beauty scoring
								</label>
								<select
									id="beauty-scoring-mode"
									class="mt-1 w-full rounded border px-3 py-2 text-gray-700 shadow"
									value={$beautyScoringModeStore}
									on:change={updateBeautyScoringMode}
									disabled={!isModerator || !canChangeStorytellerScoringSettings}
								>
									<option value="vote_divisor">Vote divisor</option>
									<option value="winner_bonus">Winner bonus</option>
								</select>
								<p class="mt-1 text-xs opacity-75">
									Vote divisor awards floor(cumulative current-game beauty votes / K). Winner bonus
									keeps the legacy top-card bonus flow.
								</p>
							</div>
						</div>
						{#if $beautyScoringModeStore === 'vote_divisor'}
							<div class="mt-3 space-y-3">
								<div>
									<label class="text-sm font-medium" for="beauty-vote-points-divisor-mode">
										Beauty vote divisor K mode
									</label>
									<select
										id="beauty-vote-points-divisor-mode"
										class="mt-1 w-full rounded border px-3 py-2 text-gray-700 shadow"
										value={$beautyVotePointsDivisorModeStore}
										on:change={updateBeautyVotePointsDivisorMode}
										disabled={!isModerator || !canChangeStorytellerScoringSettings}
									>
										<option value="manual">Manual</option>
										<option value="player_count_auto">Auto: player count</option>
										<option value="median_auto">Auto: median beauty votes / round</option>
									</select>
								</div>
								{#if $beautyVotePointsDivisorModeStore === 'manual'}
									<div>
										<label class="text-sm font-medium" for="beauty-vote-points-divisor">
											Manual beauty vote divisor K
										</label>
										<input
											id="beauty-vote-points-divisor"
											type="number"
											class="mt-1 w-full rounded border px-2 py-1 text-gray-700 shadow"
											min={BEAUTY_VOTE_POINTS_DIVISOR_MIN}
											max={BEAUTY_VOTE_POINTS_DIVISOR_MAX}
											step={BEAUTY_VOTE_POINTS_DIVISOR_STEP}
											value={formattedBeautyVotePointsDivisor}
											on:change={updateBeautyVotePointsDivisor}
											disabled={!isModerator || !canChangeStorytellerScoringSettings}
										/>
										<p class="mt-1 text-xs opacity-75">
											Range: {BEAUTY_VOTE_POINTS_DIVISOR_MIN.toFixed(
												1
											)}–{BEAUTY_VOTE_POINTS_DIVISOR_MAX.toFixed(1)}
										</p>
									</div>
								{:else if $beautyVotePointsDivisorModeStore === 'player_count_auto'}
									<div>
										<label
											class="text-sm font-medium"
											for="beauty-vote-points-divisor-player-count-base"
										>
											Player-count auto base
										</label>
										<input
											id="beauty-vote-points-divisor-player-count-base"
											type="number"
											class="mt-1 w-full rounded border px-2 py-1 text-gray-700 shadow"
											min={BEAUTY_VOTE_POINTS_DIVISOR_PLAYER_COUNT_BASE_MIN}
											max={BEAUTY_VOTE_POINTS_DIVISOR_PLAYER_COUNT_BASE_MAX}
											step="1"
											value={$beautyVotePointsDivisorPlayerCountBaseStore}
											on:change={updateBeautyVotePointsDivisorPlayerCountBase}
											disabled={!isModerator || !canChangeStorytellerScoringSettings}
										/>
										<p class="mt-1 text-xs opacity-75">
											K = round((players / {$beautyVotePointsDivisorPlayerCountBaseStore}) to 1
											decimal), then clamp to at least 1.0.
										</p>
									</div>
								{:else}
									<p class="text-xs opacity-75">
										K = median(cumulative beauty votes received by current active players) /
										completed vote-divisor rounds, rounded to 1 decimal and clamped to at least 1.0.
									</p>
								{/if}
								<p class="text-xs opacity-75">
									Effective K: {formattedEffectiveBeautyVotePointsDivisor}.
								</p>
							</div>
						{:else}
							<div class="mt-3 grid gap-3 md:grid-cols-2">
								<div>
									<label class="text-sm font-medium" for="beauty-points-bonus">
										Beauty winner bonus
									</label>
									<input
										id="beauty-points-bonus"
										type="number"
										class="mt-1 w-full rounded border px-2 py-1 text-gray-700 shadow"
										min={beautyPointsBonusMin}
										max={beautyPointsBonusMax}
										step="1"
										value={beautyPointsBonus}
										on:change={updateBeautyPointsBonus}
										disabled={!isModerator || !canChangeStorytellerScoringSettings}
									/>
									<p class="mt-1 text-xs opacity-75">
										Range: {beautyPointsBonusMin}–{beautyPointsBonusMax}
									</p>
								</div>
							</div>
						{/if}
						<label class="mt-3 flex items-start gap-3 text-sm">
							<input
								type="checkbox"
								class="mt-0.5 h-4 w-4 cursor-pointer accent-primary-500"
								checked={beautyAllowDuplicateVotes}
								on:change={updateBeautyAllowDuplicateVotes}
								disabled={!isModerator || !canChangeBeforeBeautyVotingSettings}
							/>
							<span>Allow duplicate beauty votes on the same card</span>
						</label>
						{#if $beautyScoringModeStore === 'winner_bonus'}
							<label class="mt-3 flex items-start gap-3 text-sm">
								<input
									type="checkbox"
									class="mt-0.5 h-4 w-4 cursor-pointer accent-primary-500"
									checked={beautySplitPointsOnTie}
									on:change={updateBeautySplitPointsOnTie}
									disabled={!isModerator || !canChangeStorytellerScoringSettings}
								/>
								<span>Split beauty bonus among tied owners, rounding up</span>
							</label>
						{/if}
						<label class="mt-3 flex items-start gap-3 text-sm">
							<input
								type="checkbox"
								class="mt-0.5 h-4 w-4 cursor-pointer accent-primary-500"
								checked={showPreviousResultsDuringStorytellerChoosing}
								on:change={updateShowPreviousResultsDuringStorytellerChoosing}
								disabled={!isModerator || !canChangeLiveDixitSettings}
							/>
							<span>
								Show previous results while the next storyteller is choosing and during nominations,
								with a local Previous Results / My Cards switch for active players
							</span>
						</label>
						<div class="mt-3 rounded border border-white/15 px-3 py-3">
							<p class="text-sm font-medium">Clue rating</p>
							<p class="mt-1 text-xs opacity-75">
								Optional star-vote stage after storyteller and beauty voting, before results.
							</p>
							<label class="mt-3 flex items-start gap-3 text-sm">
								<input
									type="checkbox"
									class="mt-0.5 h-4 w-4 cursor-pointer accent-primary-500"
									checked={$clueRatingEnabledStore}
									on:change={updateClueRatingEnabled}
									disabled={!isModerator || !canChangeBeforeClueRatingSettings}
								/>
								<span>Enable clue rating stage</span>
							</label>
							<div class="mt-3 grid gap-3 md:grid-cols-2">
								<div>
									<label class="text-sm font-medium" for="clue-rating-max-stars"> Max stars </label>
									<input
										id="clue-rating-max-stars"
										type="number"
										class="mt-1 w-full rounded border px-2 py-1 text-gray-700 shadow"
										min={$clueRatingMaxStarsMinStore}
										max={$clueRatingMaxStarsMaxStore}
										step="1"
										value={$clueRatingMaxStarsStore}
										on:change={updateClueRatingMaxStars}
										disabled={!isModerator || !canChangeBeforeClueRatingSettings}
									/>
									<p class="mt-1 text-xs opacity-75">
										Range: {$clueRatingMaxStarsMinStore}–{$clueRatingMaxStarsMaxStore}
									</p>
								</div>
								<div class="rounded border border-white/10 px-3 py-2 text-xs opacity-80">
									Storyteller bonus = max(round(average stars) - 1, 0).
								</div>
							</div>
						</div>
						{#if !canChangeBeforeBeautyVotingSettings}
							<p class="mt-1 text-xs opacity-70">{BEFORE_BEAUTY_VOTING_HINT}</p>
						{:else if !canChangeBeforeClueRatingSettings}
							<p class="mt-1 text-xs opacity-70">{BEFORE_CLUE_RATING_HINT}</p>
						{:else if !canChangeBeforeResultsSettings}
							<p class="mt-1 text-xs opacity-70">{BEFORE_RESULTS_HINT}</p>
						{:else if !canChangeStorytellerScoringSettings}
							<p class="mt-1 text-xs opacity-70">{LIVE_DIXIT_STAGE_HINT}</p>
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
								disabled={!isModerator || !canChangeBeforeVotingSettings}
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
										disabled={!isModerator || !canChangeBeforeVotingSettings}
									/>
									<p class="mt-1 text-xs opacity-70">Range: 0–{MAX_VOTING_WRONG_CARD_DISABLE_X}</p>
								</div>
								<p class="text-xs opacity-70">
									Editing probabilities auto-normalizes the total to 100% and switches the setting
									to Custom.
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
													!canChangeBeforeVotingSettings ||
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
													!canChangeBeforeVotingSettings ||
													normalizedVotingWrongCardDisableDistribution.length === 1}
											/>
											<span class="text-xs opacity-75">%</span>
										</div>
									</div>
								{/each}
							</div>
						</details>
						{#if !canChangeBeforeVotingSettings}
							<p class="mt-1 text-xs opacity-70">{BEFORE_VOTING_HINT}</p>
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
							<p class="mt-1 text-xs opacity-70">{STORYTELLER_CHOOSING_ONLY_HINT}</p>
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
								disabled={!isModerator || !canChangeBeforeVotingSettings}
							/>
							<span class="text-xs opacity-75">
								Range: {nominationsPerGuesserMin}–{nominationsPerGuesserMax}
							</span>
						</div>
						{#if !canChangeBeforeVotingSettings}
							<p class="mt-1 text-xs opacity-70">{BEFORE_VOTING_HINT}</p>
						{/if}
					</div>
					<div class="mt-3 rounded border border-white/20 px-2 py-2">
						<p class="block text-sm font-semibold">Round-start discards</p>
						<p class="mt-1 text-xs opacity-75">
							Pinned cards stay in hand. Unpinned cards are discarded at round start, then hands top
							up.
						</p>
						<label class="mt-2 flex items-start gap-3 text-sm">
							<input
								type="checkbox"
								class="mt-0.5 h-4 w-4 cursor-pointer accent-primary-500"
								checked={roundStartDiscardAllUnpinned}
								on:change={updateRoundStartDiscardAllUnpinned}
								disabled={!isModerator || !canChangeLiveDixitSettings}
							/>
							<span>
								<span class="font-semibold">Discard all unpinned cards at round start</span>
								<span class="block text-xs opacity-75">
									When unchecked, use the random discard count below instead.
								</span>
							</span>
						</label>
						<div class="mt-2 flex items-center gap-2">
							<input
								type="number"
								class="w-24 rounded border px-2 py-1 text-gray-700 shadow"
								min="0"
								max={roundStartDiscardCountMax}
								step="1"
								value={roundStartDiscardCount}
								on:change={updateRoundStartDiscardCount}
								disabled={!isModerator ||
									!canChangeLiveDixitSettings ||
									roundStartDiscardAllUnpinned}
							/>
							<span class="text-xs opacity-75">Range: 0–{roundStartDiscardCountMax}</span>
						</div>
						{#if !canChangeLiveDixitSettings}
							<p class="mt-1 text-xs opacity-70">{LIVE_DIXIT_STAGE_HINT}</p>
						{/if}
					</div>
					<div class="mt-3 rounded border border-white/20 px-2 py-2">
						<label class="flex items-start gap-3 text-sm">
							<input
								type="checkbox"
								class="mt-0.5 h-4 w-4 cursor-pointer accent-primary-500"
								checked={bonusCorrectGuessOnThresholdCorrectLoss}
								on:change={updateBonusCorrectGuessOnThresholdCorrectLoss}
								disabled={!isModerator || !canChangeStorytellerScoringSettings}
							/>
							<span> Give +3 correct-guess base in storyteller-loss rounds covered below </span>
						</label>
						{#if !canChangeStorytellerScoringSettings}
							<p class="mt-1 text-xs opacity-70">{LIVE_DIXIT_STAGE_HINT}</p>
						{/if}
					</div>
					<div class="mt-3 rounded border border-white/20 px-2 py-2">
						<p class="block text-sm font-semibold">Double-vote bonus</p>
						<p class="mt-1 text-xs opacity-75">
							Extra points when 2+ of your vote tokens hit the storyteller card.
						</p>
						<div class="mt-3 space-y-3">
							<div class="grid gap-2 md:grid-cols-[minmax(0,1fr)_auto] md:items-center">
								<div>
									<p class="text-sm font-medium">Normal round</p>
									<p class="text-xs opacity-75">Used whenever the storyteller wins the round.</p>
								</div>
								<div class="flex items-center gap-2">
									<input
										type="number"
										class="w-24 rounded border px-2 py-1 text-gray-700 shadow"
										min={doubleVoteBonusPointsMin}
										max={doubleVoteBonusPointsMax}
										step="1"
										value={doubleVoteBonusNormalPoints}
										on:change={updateDoubleVoteBonusNormalPoints}
										disabled={!isModerator || !canChangeStorytellerScoringSettings}
									/>
									<span class="text-xs opacity-75">
										Range: {doubleVoteBonusPointsMin}–{doubleVoteBonusPointsMax}
									</span>
								</div>
							</div>
							<div class="rounded border border-white/10 px-3 py-3">
								<div class="grid gap-3 md:grid-cols-[minmax(0,1fr)_auto_auto] md:items-center">
									<div>
										<p class="text-sm font-medium">Too many guessed wrong</p>
										<p class="text-xs opacity-75">
											Storyteller-loss rounds where too many players missed the storyteller card.
										</p>
									</div>
									<label class="flex items-center gap-2 text-sm">
										<input
											type="checkbox"
											class="h-4 w-4 cursor-pointer accent-primary-500"
											checked={doubleVoteBonusTooManyWrongFollowsNormal}
											on:change={updateDoubleVoteBonusTooManyWrongFollowsNormal}
											disabled={!isModerator || !canChangeStorytellerScoringSettings}
										/>
										<span>Follow normal</span>
									</label>
									<input
										type="number"
										class="w-24 rounded border px-2 py-1 text-gray-700 shadow"
										min={doubleVoteBonusPointsMin}
										max={doubleVoteBonusPointsMax}
										step="1"
										value={effectiveDoubleVoteBonusTooManyWrongPoints}
										on:change={updateDoubleVoteBonusTooManyWrongPoints}
										disabled={!isModerator ||
											!canChangeStorytellerScoringSettings ||
											doubleVoteBonusTooManyWrongFollowsNormal}
									/>
								</div>
							</div>
							<div class="rounded border border-white/10 px-3 py-3">
								<div class="grid gap-3 md:grid-cols-[minmax(0,1fr)_auto_auto] md:items-center">
									<div>
										<p class="text-sm font-medium">Too many guessed correctly</p>
										<p class="text-xs opacity-75">
											Storyteller-loss rounds where too many players found the storyteller card.
										</p>
									</div>
									<label class="flex items-center gap-2 text-sm">
										<input
											type="checkbox"
											class="h-4 w-4 cursor-pointer accent-primary-500"
											checked={doubleVoteBonusTooManyCorrectFollowsNormal}
											on:change={updateDoubleVoteBonusTooManyCorrectFollowsNormal}
											disabled={!isModerator || !canChangeStorytellerScoringSettings}
										/>
										<span>Follow normal</span>
									</label>
									<input
										type="number"
										class="w-24 rounded border px-2 py-1 text-gray-700 shadow"
										min={doubleVoteBonusPointsMin}
										max={doubleVoteBonusPointsMax}
										step="1"
										value={effectiveDoubleVoteBonusTooManyCorrectPoints}
										on:change={updateDoubleVoteBonusTooManyCorrectPoints}
										disabled={!isModerator ||
											!canChangeStorytellerScoringSettings ||
											doubleVoteBonusTooManyCorrectFollowsNormal}
									/>
								</div>
							</div>
						</div>
						{#if !canChangeStorytellerScoringSettings}
							<p class="mt-2 text-xs opacity-70">{LIVE_DIXIT_STAGE_HINT}</p>
						{/if}
					</div>
					<div class="mt-3 rounded border border-white/20 px-2 py-2">
						<label class="flex items-start gap-3 text-sm">
							<input
								type="checkbox"
								class="mt-0.5 h-4 w-4 cursor-pointer accent-primary-500"
								checked={bonusThresholdLossTogglesApplyToAllStorytellerLossRounds}
								on:change={updateBonusThresholdLossTogglesApplyToAllStorytellerLossRounds}
								disabled={!isModerator || !canChangeStorytellerScoringSettings}
							/>
							<span> Apply the correct-guess +3 bonus above in all storyteller-loss rounds </span>
						</label>
						<p class="mt-1 text-xs opacity-70">
							Includes rounds where the storyteller loses because too many guessers were wrong.
						</p>
						{#if !canChangeStorytellerScoringSettings}
							<p class="mt-1 text-xs opacity-70">{LIVE_DIXIT_STAGE_HINT}</p>
						{/if}
					</div>
				{/if}
			{/if}
		</details>
	{/if}
</div>
