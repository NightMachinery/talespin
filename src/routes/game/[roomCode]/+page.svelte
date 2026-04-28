<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { get } from 'svelte/store';

	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import { getToastStore } from '@skeletonlabs/skeleton';

	import {
		applyRoomLeaderboardViewModeDefault,
		setBeautyScoringConfig,
		beautyLeaderboardPointChange,
		memberBeautyPoints,
		refreshMostBeautifulStats,
		resetMostBeautifulClientState,
		setMostBeautifulRoom,
		storytellerLeaderboardPointChange
	} from '$lib/mostBeautiful';
	import {
		clearClueRatingResults,
		resetClueRatingClientState,
		setClueRatingResults,
		setClueRatingRoomState
	} from '$lib/clueRating';
	import {
		readRoomMigrationOverride,
		resetCurrentRoomMigration,
		setCurrentRoomAuthId,
		setCurrentRoomMigrationRoom,
		setCurrentRoomPassword
	} from '$lib/deviceMigration';
	import { derivePerViewerVotingLayout } from '$lib/dixitVotingLayout';
	import {
		clearAssignedRoomName,
		getJoinNameForRoom,
		nameStore,
		playerTokenStore,
		setAssignedRoomName
	} from '$lib/store';
	import {
		playScoutTurnCue,
		playStageChangeCue,
		unlockStageChangeAudio
	} from '$lib/stageChangeAudio';
	import {
		getCueVisualTone,
		isStageCueStage,
		type CueVisualTone,
		type StageCueStage
	} from '$lib/stageCues';
	import type {
		BeautyResultsDisplayMode,
		BeautyScoringMode,
		BeautyVotePointsDivisorMode,
		GameMode,
		LeaderboardViewMode,
		ObserverInfo,
		PreviousDixitResultsView,
		PlayerInfo,
		StellaQueuedRevealMode,
		WinCondition
	} from '$lib/types';
	import { stageChangeSoundCuesEnabled, stageChangeVisualCuesEnabled } from '$lib/viewOptions';
	import GameServer from '$lib/gameServer';
	import { leaderboardRoundHistory } from '$lib/leaderboard';
	import { DEFAULT_VOTING_WRONG_CARD_DISABLE_DISTRIBUTION } from '$lib/votingWrongCardDisableDistribution';

	import Joining from './Joining.svelte';
	import ActiveChooses from './ActiveChooses.svelte';
	import PlayersChoose from './PlayersChoose.svelte';
	import Voting from './Voting.svelte';
	import BeautyVoting from './BeautyVoting.svelte';
	import ClueRating from './ClueRating.svelte';
	import Results from './Results.svelte';
	import BeautyResults from './BeautyResults.svelte';
	import Paused from './Paused.svelte';
	import End from './End.svelte';
	import ScoreCheatsheet from './ScoreCheatsheet.svelte';
	import StellaAssociate from './StellaAssociate.svelte';
	import StellaReveal from './StellaReveal.svelte';
	import StellaResults from './StellaResults.svelte';

	// connection information
	let name = '';
	let preferredName = '';
	let token = '';
	let roomCode = '';
	let gameServer: GameServer;
	let rejoin = false;
	let roomPassword = '';
	let roomAuthIdOverride = '';
	let lastJoinAttemptName = '';
	let migrationJoinError = '';

	// game state
	let players: { [key: string]: PlayerInfo } = {};
	let observers: { [key: string]: ObserverInfo } = {};
	let stage: string = 'Joining';
	let gameMode: GameMode = 'dixit_plus';
	let creator = '';
	let moderators: string[] = [];
	let allowNewPlayersMidgame = true;
	let pausedReason = '';
	let storytellerLossComplement = 0;
	let storytellerLossComplementMin = 0;
	let storytellerLossComplementMax = 0;
	let storytellerLossComplementAuto = true;
	let storytellerPoolEnabled = false;
	let storytellerPoolActive = false;
	let storytellerPoolPlayers: string[] = [];
	let storytellerSuccessPoints = 3;
	let storytellerSuccessPointsMin = 0;
	let storytellerSuccessPointsMax = 10;
	let votesPerGuesser = 1;
	let votesPerGuesserMin = 1;
	let votesPerGuesserMax = 1;
	let beautyEnabled = false;
	let beautyVotesPerPlayer = 2;
	let beautyVotesPerPlayerMin = 1;
	let beautyVotesPerPlayerMax = 1;
	let beautyAllowDuplicateVotes = false;
	let beautySplitPointsOnTie = true;
	let beautyPointsBonus = 2;
	let beautyScoringMode: BeautyScoringMode = 'vote_divisor';
	let beautyVotePointsDivisorMode: BeautyVotePointsDivisorMode = 'manual';
	let beautyVotePointsDivisor = 3;
	let beautyVotePointsDivisorPlayerCountBase = 4;
	let beautyVotePointsDivisorEffective: number | null = 3;
	let beautyPointsBonusMin = 0;
	let beautyPointsBonusMax = 10;
	let beautyResultsDisplayMode: BeautyResultsDisplayMode = 'combined';
	let showPreviousResultsDuringStorytellerChoosing = true;
	let randomizeVotingCardOrderPerViewer = false;
	let cardsPerHand = 12;
	let cardsPerHandMin = 1;
	let cardsPerHandMax = 18;
	let nominationsPerGuesser = 1;
	let nominationsPerGuesserMin = 1;
	let nominationsPerGuesserMax = 1;
	let bonusCorrectGuessOnThresholdCorrectLoss = true;
	let doubleVoteBonusNormalPoints = 1;
	let doubleVoteBonusTooManyWrongPoints = 1;
	let doubleVoteBonusTooManyWrongFollowsNormal = true;
	let doubleVoteBonusTooManyCorrectPoints = 1;
	let doubleVoteBonusTooManyCorrectFollowsNormal = true;
	let doubleVoteBonusPointsMin = 0;
	let doubleVoteBonusPointsMax = 10;
	let bonusThresholdLossTogglesApplyToAllStorytellerLossRounds = true;
	let showVotingCardNumbers = true;
	let roundStartDiscardAllUnpinned = true;
	let roundStartDiscardCount = 3;
	let hintChoosingTimerEnabled = true;
	let hintChoosingTimerDurationS = 60;
	let forceHintChoosingTimer = false;
	let cardChoosingTimerEnabled = true;
	let cardChoosingTimerDurationS = 30;
	let votingTimerEnabled = true;
	let votingTimerDurationS = 180;
	let beautyTimerEnabled = true;
	let beautyTimerDurationS = 60;
	let forceCardChoosingTimer = false;
	let forceVotingTimer = false;
	let forceBeautyTimer = false;
	let serverTimeMs: number | null = null;
	let currentStageDeadlineS: number | null = null;
	let votingWrongCardDisableDistribution = [...DEFAULT_VOTING_WRONG_CARD_DISABLE_DISTRIBUTION];
	let stellaBoardSize = 15;
	let stellaBoardSizeMin = 1;
	let stellaBoardSizeMax = 100;
	let stellaSelectionMin = 1;
	let stellaSelectionMax = 10;
	let stellaSelectionCountMin = 1;
	let stellaSelectionCountMax = 15;
	let stellaActiveClue = '';
	let stellaWordPackWords: string[] = [];
	let stellaWordPackPresetNames: string[] = [];
	let stellaSelectedWordPackName = '';
	let stellaWordPackIsUnsaved = false;
	let stellaQueueDuringAssociation = true;
	let stellaQueuedRevealMode: StellaQueuedRevealMode = 'animated';
	let stellaScoutTimerEnabled = true;
	let stellaScoutTimerDurationS = 10;
	let forceStellaScoutTimer = false;
	let stellaDarkPlayer = '';
	let activePlayer = '';
	let description = '';
	let roundNum = 0;
	let cardsRemaining = 0;
	let deckRefillCount = 0;
	let deckRefillFlashToken = 0;
	let hasReceivedRoomState = false;
	let leaderboardViewModeDefault: LeaderboardViewMode = 'combined';
	let leaderboardExcludeBeautyDefaultVersion = 0;
	let winCondition: WinCondition = {
		mode: 'points',
		target_points: 10
	};

	// UI state
	let stageImages: string[] = [];
	let displayImages: string[] = [];
	let myHandImages: string[] = [];
	let pinnedCards: string[] = [];
	let displayImageBadgeLabels: number[] = [];
	let votingDisabledCards: string[] = [];
	let beautyDisabledCards: string[] = [];
	let storytellerChosenCard = '';
	let playersChooseSelectedCards: string[] = [];
	let previousDixitResults: PreviousDixitResultsView | null = null;
	let votingLayoutSeed: string | null = null;
	let stageVisualCueVisible = false;
	let stageVisualCueKey = 0;
	let activeStageVisualCueTone: CueVisualTone = 'choose';
	let stageVisualCueTimeout: number | undefined;

	// results
	let playerToCurrentCards: { [key: string]: string[] } = {};
	let playerToVotes: { [key: string]: string[] } = {};
	let playerToBeautyVotes: { [key: string]: string[] } = {};
	let activeCard = '';
	let pointChange: { [key: string]: number } = {};
	let storytellerPointChange: { [key: string]: number } = {};
	let beautyPointChange: { [key: string]: number } = {};
	let beautyVoteTotals: { [key: string]: number } = {};
	let beautyWinningCards: string[] = [];
	let stellaSelectedCards: string[] = [];
	let stellaSelectedCounts: { [key: string]: number } = {};
	let stellaRevealedCards: string[] = [];
	let stellaRevealedCardChoosers: { [key: string]: string[] } = {};
	let stellaCardPoints: { [key: string]: number } = {};
	let previousScoutTurnKey = '';
	let hasLoadedMostBeautifulStats = false;
	let clueRatingStageMaxStars = 3;

	// store
	let toastStore = getToastStore();
	nameStore.subscribe((value) => {
		preferredName = value;
		if (name === '') {
			name = value;
		}
	});
	playerTokenStore.subscribe((value) => (token = value));

	function setStage(nextStage: string, { suppressCue = false } = {}) {
		const previousStage = stage;
		stage = nextStage;
		if (suppressCue || nextStage === previousStage || !isStageCueStage(nextStage)) return;
		triggerStageCue(nextStage);
	}

	function maybeUnlockStageChangeAudio() {
		if (!get(stageChangeSoundCuesEnabled)) return;
		void unlockStageChangeAudio();
	}

	function triggerStageCue(nextStage: StageCueStage) {
		if (get(stageChangeSoundCuesEnabled)) {
			void playStageChangeCue(nextStage);
		}

		if (get(stageChangeVisualCuesEnabled)) {
			triggerStageVisualCue(nextStage);
		}
	}

	function triggerStageVisualCue(nextStage: StageCueStage) {
		const visualTone = getCueVisualTone(nextStage);
		if (!visualTone) return;

		activeStageVisualCueTone = visualTone;
		stageVisualCueVisible = true;
		stageVisualCueKey += 1;

		if (stageVisualCueTimeout) {
			clearTimeout(stageVisualCueTimeout);
		}

		stageVisualCueTimeout = window.setTimeout(() => {
			stageVisualCueVisible = false;
		}, 1100);
	}

	function syncStageTimer(
		nextServerTimeMs: number | null | undefined,
		nextDeadlineS: number | null | undefined
	) {
		serverTimeMs = nextServerTimeMs ?? null;
		currentStageDeadlineS = nextDeadlineS ?? null;
	}

	function clearStageTimer() {
		currentStageDeadlineS = null;
	}

	$: scoutTurnCueKey =
		stage === 'StellaReveal' && activePlayer === name
			? `${roundNum}:${activePlayer}:${stellaRevealedCards.length}`
			: '';
	$: if (scoutTurnCueKey === '') {
		previousScoutTurnKey = '';
	} else if (scoutTurnCueKey !== previousScoutTurnKey && get(stageChangeSoundCuesEnabled)) {
		previousScoutTurnKey = scoutTurnCueKey;
		void playScoutTurnCue();
	}
	$: {
		const shouldRandomizeVotingLayout =
			gameMode === 'dixit_plus' &&
			(stage === 'Voting' || stage === 'BeautyVoting') &&
			randomizeVotingCardOrderPerViewer;
		if (shouldRandomizeVotingLayout) {
			const derivedLayout = derivePerViewerVotingLayout(stageImages, name, votingLayoutSeed, true);
			displayImages = derivedLayout.images;
			displayImageBadgeLabels = derivedLayout.badgeLabels;
		} else {
			displayImages = [...stageImages];
			displayImageBadgeLabels = stageImages.map((_, index) => index + 1);
		}
	}

	function usingRoomAuthOverride() {
		return roomAuthIdOverride !== '';
	}

	function clearStoredAssignedName() {
		if (usingRoomAuthOverride()) {
			return;
		}

		if (roomCode !== '') {
			clearAssignedRoomName(roomCode);
		}
	}

	function persistAssignedName(assignedName: string) {
		name = assignedName;
		if (usingRoomAuthOverride()) {
			return;
		}

		if (roomCode === '') {
			return;
		}

		if (assignedName !== '' && assignedName !== preferredName) {
			setAssignedRoomName(roomCode, token, assignedName);
		} else {
			clearStoredAssignedName();
		}
	}

	function joinCurrentRoom(includePassword = false) {
		const joinToken = usingRoomAuthOverride() ? roomAuthIdOverride : token;
		const joinName = usingRoomAuthOverride()
			? ''
			: getJoinNameForRoom(roomCode, preferredName, token);
		lastJoinAttemptName = joinName;
		name = joinName;
		migrationJoinError = '';
		gameServer.joinRoom(roomCode, joinName, joinToken, includePassword ? roomPassword : undefined);
	}

	onDestroy(() => {
		if (stageVisualCueTimeout) {
			clearTimeout(stageVisualCueTimeout);
		}
		resetMostBeautifulClientState();
		resetClueRatingClientState();
		leaderboardRoundHistory.set([]);
		resetCurrentRoomMigration();
		if (gameServer) {
			rejoin = false;
			gameServer.close();
		}
	});

	onMount(() => {
		roomCode = $page.params.roomCode;
		resetCurrentRoomMigration();
		setCurrentRoomMigrationRoom(roomCode);
		if (typeof window !== 'undefined') {
			const url = new URL(window.location.href);
			const roomMigrationOverride = readRoomMigrationOverride(url);
			roomAuthIdOverride = roomMigrationOverride.roomAuthId;
			roomPassword =
				roomMigrationOverride.roomPassword ||
				window.sessionStorage.getItem(`room_password_${roomCode}`) ||
				'';
		}
		setCurrentRoomPassword(roomPassword);
		setCurrentRoomAuthId('');

		setMostBeautifulRoom(roomCode);
		name = usingRoomAuthOverride() ? '' : getJoinNameForRoom(roomCode, preferredName, token);
		if (!usingRoomAuthOverride() && name === '') {
			goto(`/?room=${roomCode}`);
			return;
		}

		if (typeof window !== 'undefined') {
			window.addEventListener('pointerdown', maybeUnlockStageChangeAudio, { passive: true });
			window.addEventListener('keydown', maybeUnlockStageChangeAudio);
		}

		gameServer = new GameServer();
		joinCurrentRoom(true);
		gameServer.onclose(() => {
			if (rejoin) {
				joinCurrentRoom();
			}
		});
		// eslint-disable-next-line @typescript-eslint/no-explicit-any
		gameServer.addMsgHandler((data: any) => {
			console.log(data);

			if (data.JoinedAs) {
				migrationJoinError = '';
				const assignedName = (data.JoinedAs.name || '').trim();
				const joinedRoomAuthId = (data.JoinedAs.room_auth_id || '').trim();
				if (joinedRoomAuthId !== '') {
					setCurrentRoomAuthId(joinedRoomAuthId);
				}
				if (assignedName !== '') {
					const previousAttemptName = lastJoinAttemptName;
					persistAssignedName(assignedName);
					if (previousAttemptName !== '' && assignedName !== previousAttemptName) {
						toastStore.trigger({
							message: `🙋 Name already taken, joined as ${assignedName}`,
							autohide: true,
							timeout: 3000
						});
					}
				}
			} else if (data.RoomState) {
				migrationJoinError = '';
				if (!hasReceivedRoomState && roomPassword !== '' && typeof window !== 'undefined') {
					window.sessionStorage.removeItem(`room_password_${roomCode}`);
				}
				const previousDeckRefillCount = deckRefillCount;
				players = data.RoomState.players;
				observers = data.RoomState.observers || {};
				gameMode = data.RoomState.game_mode || 'dixit_plus';
				creator = data.RoomState.creator || '';
				moderators = data.RoomState.moderators || [];
				setStage(data.RoomState.stage, { suppressCue: !hasReceivedRoomState });
				allowNewPlayersMidgame = data.RoomState.allow_new_players_midgame ?? true;
				pausedReason = data.RoomState.paused_reason || '';
				storytellerLossComplement = data.RoomState.storyteller_loss_complement ?? 0;
				storytellerLossComplementMin = data.RoomState.storyteller_loss_complement_min ?? 0;
				storytellerLossComplementMax = data.RoomState.storyteller_loss_complement_max ?? 0;
				storytellerLossComplementAuto = data.RoomState.storyteller_loss_complement_auto ?? true;
				storytellerPoolEnabled = data.RoomState.storyteller_pool_enabled ?? false;
				storytellerPoolActive = data.RoomState.storyteller_pool_active ?? false;
				storytellerPoolPlayers = data.RoomState.storyteller_pool_players ?? [];
				storytellerSuccessPoints = data.RoomState.storyteller_success_points ?? 3;
				storytellerSuccessPointsMin = data.RoomState.storyteller_success_points_min ?? 0;
				storytellerSuccessPointsMax = data.RoomState.storyteller_success_points_max ?? 10;
				votesPerGuesser = data.RoomState.votes_per_guesser ?? 1;
				votesPerGuesserMin = data.RoomState.votes_per_guesser_min ?? 1;
				votesPerGuesserMax = data.RoomState.votes_per_guesser_max ?? 1;
				beautyEnabled = data.RoomState.beauty_enabled ?? false;
				beautyVotesPerPlayer = data.RoomState.beauty_votes_per_player ?? 2;
				beautyVotesPerPlayerMin = data.RoomState.beauty_votes_per_player_min ?? 1;
				beautyVotesPerPlayerMax = data.RoomState.beauty_votes_per_player_max ?? 1;
				beautyAllowDuplicateVotes = data.RoomState.beauty_allow_duplicate_votes ?? false;
				beautySplitPointsOnTie = data.RoomState.beauty_split_points_on_tie ?? true;
				beautyPointsBonus = data.RoomState.beauty_points_bonus ?? 2;
				beautyScoringMode = data.RoomState.beauty_scoring_mode ?? 'vote_divisor';
				beautyVotePointsDivisorMode = data.RoomState.beauty_vote_points_divisor_mode ?? 'manual';
				beautyVotePointsDivisor = data.RoomState.beauty_vote_points_divisor ?? 3;
				beautyVotePointsDivisorPlayerCountBase =
					data.RoomState.beauty_vote_points_divisor_player_count_base ?? 4;
				beautyVotePointsDivisorEffective =
					data.RoomState.beauty_vote_points_divisor_effective === undefined
						? beautyVotePointsDivisor
						: data.RoomState.beauty_vote_points_divisor_effective;
				setBeautyScoringConfig(
					beautyScoringMode,
					beautyVotePointsDivisor,
					beautyVotePointsDivisorMode,
					beautyVotePointsDivisorPlayerCountBase,
					beautyVotePointsDivisorEffective
				);
				beautyPointsBonusMin = data.RoomState.beauty_points_bonus_min ?? 0;
				beautyPointsBonusMax = data.RoomState.beauty_points_bonus_max ?? 10;
				beautyResultsDisplayMode = data.RoomState.beauty_results_display_mode ?? 'combined';
				showPreviousResultsDuringStorytellerChoosing =
					data.RoomState.show_previous_results_during_storyteller_choosing ?? true;
				randomizeVotingCardOrderPerViewer =
					data.RoomState.randomize_voting_card_order_per_viewer ?? false;
				votingLayoutSeed = data.RoomState.voting_layout_seed ?? null;
				previousDixitResults = data.RoomState.previous_dixit_results ?? null;
				cardsPerHand = data.RoomState.cards_per_hand ?? 12;
				cardsPerHandMin = data.RoomState.cards_per_hand_min ?? 1;
				cardsPerHandMax = data.RoomState.cards_per_hand_max ?? 18;
				nominationsPerGuesser = data.RoomState.nominations_per_guesser ?? 1;
				nominationsPerGuesserMin = data.RoomState.nominations_per_guesser_min ?? 1;
				nominationsPerGuesserMax = data.RoomState.nominations_per_guesser_max ?? 1;
				bonusCorrectGuessOnThresholdCorrectLoss =
					data.RoomState.bonus_correct_guess_on_threshold_correct_loss ?? true;
				doubleVoteBonusNormalPoints = data.RoomState.double_vote_bonus_normal_points ?? 1;
				doubleVoteBonusTooManyWrongPoints =
					data.RoomState.double_vote_bonus_too_many_wrong_points ?? doubleVoteBonusNormalPoints;
				doubleVoteBonusTooManyWrongFollowsNormal =
					data.RoomState.double_vote_bonus_too_many_wrong_follows_normal ?? true;
				doubleVoteBonusTooManyCorrectPoints =
					data.RoomState.double_vote_bonus_too_many_correct_points ?? doubleVoteBonusNormalPoints;
				doubleVoteBonusTooManyCorrectFollowsNormal =
					data.RoomState.double_vote_bonus_too_many_correct_follows_normal ?? true;
				doubleVoteBonusPointsMin = data.RoomState.double_vote_bonus_points_min ?? 0;
				doubleVoteBonusPointsMax = data.RoomState.double_vote_bonus_points_max ?? 10;
				bonusThresholdLossTogglesApplyToAllStorytellerLossRounds =
					data.RoomState.bonus_threshold_loss_toggles_apply_to_all_storyteller_loss_rounds ?? true;
				showVotingCardNumbers = data.RoomState.show_voting_card_numbers ?? true;
				roundStartDiscardAllUnpinned = data.RoomState.round_start_discard_all_unpinned ?? true;
				roundStartDiscardCount = data.RoomState.round_start_discard_count ?? 3;
				hintChoosingTimerEnabled = data.RoomState.hint_choosing_timer_enabled ?? true;
				hintChoosingTimerDurationS = data.RoomState.hint_choosing_timer_duration_s ?? 60;
				forceHintChoosingTimer = data.RoomState.force_hint_choosing_timer ?? false;
				cardChoosingTimerEnabled = data.RoomState.card_choosing_timer_enabled ?? true;
				cardChoosingTimerDurationS = data.RoomState.card_choosing_timer_duration_s ?? 30;
				votingTimerEnabled = data.RoomState.voting_timer_enabled ?? true;
				votingTimerDurationS = data.RoomState.voting_timer_duration_s ?? 180;
				beautyTimerEnabled = data.RoomState.beauty_timer_enabled ?? true;
				beautyTimerDurationS = data.RoomState.beauty_timer_duration_s ?? 60;
				forceCardChoosingTimer = data.RoomState.force_card_choosing_timer ?? false;
				forceVotingTimer = data.RoomState.force_voting_timer ?? false;
				forceBeautyTimer = data.RoomState.force_beauty_timer ?? false;
				setClueRatingRoomState({
					enabled: data.RoomState.clue_rating_enabled ?? false,
					maxStars: data.RoomState.clue_rating_max_stars ?? 3,
					minStars: data.RoomState.clue_rating_max_stars_min ?? 1,
					maxStarsLimit: data.RoomState.clue_rating_max_stars_max ?? 10,
					timerEnabled: data.RoomState.clue_rating_timer_enabled ?? true,
					timerDurationS: data.RoomState.clue_rating_timer_duration_s ?? 20,
					forceTimer: data.RoomState.force_clue_rating_timer ?? false,
					memberAverage: data.RoomState.member_to_clue_rating_average ?? {},
					memberRounds: data.RoomState.member_to_clue_rating_rounds ?? {}
				});
				syncStageTimer(data.RoomState.server_time_ms, data.RoomState.current_stage_deadline_s);
				votingWrongCardDisableDistribution = data.RoomState
					.voting_wrong_card_disable_distribution ?? [
					...DEFAULT_VOTING_WRONG_CARD_DISABLE_DISTRIBUTION
				];
				memberBeautyPoints.set(data.RoomState.member_to_beauty_points ?? {});
				leaderboardViewModeDefault = data.RoomState.leaderboard_view_mode_default ?? 'combined';
				leaderboardExcludeBeautyDefaultVersion =
					data.RoomState.leaderboard_view_mode_default_version ?? 0;
				applyRoomLeaderboardViewModeDefault(
					roomCode,
					leaderboardViewModeDefault,
					leaderboardExcludeBeautyDefaultVersion
				);
				leaderboardRoundHistory.set(data.RoomState.leaderboard_round_history ?? []);
				stellaBoardSize = data.RoomState.stella_board_size ?? 15;
				stellaBoardSizeMin = data.RoomState.stella_board_size_min ?? 1;
				stellaBoardSizeMax = data.RoomState.stella_board_size_max ?? 100;
				stellaSelectionMin = data.RoomState.stella_selection_min ?? 1;
				stellaSelectionMax = data.RoomState.stella_selection_max ?? 10;
				stellaSelectionCountMin = data.RoomState.stella_selection_count_min ?? 1;
				stellaSelectionCountMax = data.RoomState.stella_selection_count_max ?? 15;
				stellaQueueDuringAssociation = data.RoomState.stella_queue_during_association ?? true;
				stellaQueuedRevealMode = data.RoomState.stella_queued_reveal_mode ?? 'animated';
				stellaScoutTimerEnabled = data.RoomState.stella_scout_timer_enabled ?? true;
				stellaScoutTimerDurationS = data.RoomState.stella_scout_timer_duration_s ?? 10;
				forceStellaScoutTimer = data.RoomState.force_stella_scout_timer ?? false;
				stellaActiveClue = data.RoomState.stella_active_clue || '';
				stellaWordPackWords = data.RoomState.stella_word_pack_words || [];
				stellaWordPackPresetNames = data.RoomState.stella_word_pack_preset_names || [];
				stellaSelectedWordPackName = data.RoomState.stella_selected_word_pack_name || '';
				stellaWordPackIsUnsaved = data.RoomState.stella_word_pack_is_unsaved ?? false;
				stellaDarkPlayer = data.RoomState.stella_dark_player || '';
				activePlayer = data.RoomState.active_player || '';
				roundNum = data.RoomState.round;
				cardsRemaining = data.RoomState.cards_remaining || 0;
				deckRefillCount = data.RoomState.deck_refill_count || 0;
				if (data.RoomState.win_condition) {
					winCondition = data.RoomState.win_condition;
				}
				if (hasReceivedRoomState && deckRefillCount > previousDeckRefillCount) {
					deckRefillFlashToken += 1;
				}
				hasReceivedRoomState = true;
				if (gameMode === 'dixit_plus' && !hasLoadedMostBeautifulStats) {
					hasLoadedMostBeautifulStats = true;
					void refreshMostBeautifulStats();
				}
				if (!rejoin) {
					toastStore.trigger({
						message: '👋 Connected to room!',
						autohide: true,
						timeout: 2500
					});
					rejoin = true;
				}
			} else if (data.StartRound) {
				syncStageTimer(data.StartRound.server_time_ms, data.StartRound.current_stage_deadline_s);
				setStage('ActiveChooses');
				votingLayoutSeed = null;
				stageImages = data.StartRound.hand;
				myHandImages = data.StartRound.hand || [];
				pinnedCards = data.StartRound.pinned_cards || [];
				playerToVotes = {};
				playerToBeautyVotes = {};
				storytellerPointChange = {};
				beautyPointChange = {};
				storytellerLeaderboardPointChange.set({});
				beautyLeaderboardPointChange.set({});
				beautyVoteTotals = {};
				beautyWinningCards = [];
				storytellerChosenCard = '';
				playersChooseSelectedCards = [];
				clearClueRatingResults();
			} else if (data.PlayersChoose) {
				syncStageTimer(
					data.PlayersChoose.server_time_ms,
					data.PlayersChoose.current_stage_deadline_s
				);
				setStage('PlayersChoose');
				votingLayoutSeed = null;
				stageImages = data.PlayersChoose.hand;
				myHandImages = data.PlayersChoose.hand || [];
				pinnedCards = data.PlayersChoose.pinned_cards || [];
				description = data.PlayersChoose.description;
				playerToVotes = {};
				playerToBeautyVotes = {};
				storytellerPointChange = {};
				beautyPointChange = {};
				storytellerLeaderboardPointChange.set({});
				beautyLeaderboardPointChange.set({});
				beautyVoteTotals = {};
				beautyWinningCards = [];
				storytellerChosenCard = data.PlayersChoose.chosen_card || '';
				playersChooseSelectedCards = data.PlayersChoose.selected_cards || [];
				clearClueRatingResults();
			} else if (data.BeginVoting) {
				syncStageTimer(data.BeginVoting.server_time_ms, data.BeginVoting.current_stage_deadline_s);
				setStage('Voting');
				votingLayoutSeed = data.BeginVoting.voting_layout_seed || votingLayoutSeed;
				stageImages = data.BeginVoting.center_cards;
				myHandImages = data.BeginVoting.hand || [];
				pinnedCards = data.BeginVoting.pinned_cards || [];
				description = data.BeginVoting.description;
				votingDisabledCards = data.BeginVoting.disabled_cards || [];
				votesPerGuesser = data.BeginVoting.votes_per_guesser ?? votesPerGuesser;
				beautyDisabledCards = [];
				playerToBeautyVotes = {};
				beautyVoteTotals = {};
				beautyWinningCards = [];
				storytellerLeaderboardPointChange.set({});
				beautyLeaderboardPointChange.set({});
				storytellerChosenCard = '';
				playersChooseSelectedCards = [];
				clearClueRatingResults();
			} else if (data.BeginBeautyVoting) {
				syncStageTimer(
					data.BeginBeautyVoting.server_time_ms,
					data.BeginBeautyVoting.current_stage_deadline_s
				);
				setStage('BeautyVoting');
				votingLayoutSeed = data.BeginBeautyVoting.voting_layout_seed || votingLayoutSeed;
				stageImages = data.BeginBeautyVoting.center_cards;
				myHandImages = data.BeginBeautyVoting.hand || [];
				pinnedCards = data.BeginBeautyVoting.pinned_cards || [];
				description = data.BeginBeautyVoting.description;
				beautyDisabledCards = data.BeginBeautyVoting.disabled_cards || [];
				beautyVotesPerPlayer = data.BeginBeautyVoting.votes_per_player ?? beautyVotesPerPlayer;
				beautyAllowDuplicateVotes =
					data.BeginBeautyVoting.allow_duplicate_votes ?? beautyAllowDuplicateVotes;
				storytellerLeaderboardPointChange.set({});
				beautyLeaderboardPointChange.set({});
				votingDisabledCards = [];
				clearClueRatingResults();
			} else if (data.BeginClueRating) {
				syncStageTimer(
					data.BeginClueRating.server_time_ms,
					data.BeginClueRating.current_stage_deadline_s
				);
				setStage('ClueRating');
				description = data.BeginClueRating.description || description;
				myHandImages = data.BeginClueRating.hand || [];
				pinnedCards = data.BeginClueRating.pinned_cards || [];
				clueRatingStageMaxStars = data.BeginClueRating.max_stars ?? clueRatingStageMaxStars;
				storytellerLeaderboardPointChange.set({});
				beautyLeaderboardPointChange.set({});
				clearClueRatingResults();
			} else if (data.Results) {
				syncStageTimer(data.Results.server_time_ms, data.Results.current_stage_deadline_s);
				setStage('Results');
				clearStageTimer();
				playerToCurrentCards = data.Results.player_to_current_cards || {};
				stageImages = data.Results.center_cards || Object.values(playerToCurrentCards).flat();
				myHandImages = data.Results.hand || [];
				pinnedCards = data.Results.pinned_cards || [];
				playerToVotes = data.Results.player_to_votes || {};
				playerToBeautyVotes = data.Results.player_to_beauty_votes || {};
				activeCard = data.Results.active_card;
				pointChange = data.Results.point_change;
				storytellerPointChange = data.Results.storyteller_point_change || {};
				beautyPointChange = data.Results.beauty_point_change || {};
				storytellerLeaderboardPointChange.set(storytellerPointChange);
				beautyLeaderboardPointChange.set(beautyPointChange);
				beautyVoteTotals = data.Results.beauty_vote_totals || {};
				beautyWinningCards = data.Results.beauty_winning_cards || [];
				setClueRatingResults({
					playerRatings: data.Results.player_to_clue_rating || {},
					average:
						data.Results.clue_rating_average === undefined
							? null
							: data.Results.clue_rating_average,
					count: data.Results.clue_rating_count ?? 0,
					bonus: data.Results.clue_rating_bonus ?? 0
				});
				votingDisabledCards = [];
				beautyDisabledCards = [];
				storytellerChosenCard = '';
				if (gameMode === 'dixit_plus') {
					void refreshMostBeautifulStats();
				}
			} else if (data.BeautyResults) {
				syncStageTimer(
					data.BeautyResults.server_time_ms,
					data.BeautyResults.current_stage_deadline_s
				);
				setStage('BeautyResults');
				clearStageTimer();
				playerToCurrentCards = data.BeautyResults.player_to_current_cards || {};
				stageImages = data.BeautyResults.center_cards || Object.values(playerToCurrentCards).flat();
				myHandImages = data.BeautyResults.hand || [];
				pinnedCards = data.BeautyResults.pinned_cards || [];
				playerToBeautyVotes = data.BeautyResults.player_to_beauty_votes || {};
				pointChange = data.BeautyResults.point_change || {};
				storytellerPointChange = {};
				beautyPointChange = data.BeautyResults.point_change || {};
				storytellerLeaderboardPointChange.set({});
				beautyLeaderboardPointChange.set(beautyPointChange);
				beautyVoteTotals = data.BeautyResults.beauty_vote_totals || {};
				beautyWinningCards = data.BeautyResults.beauty_winning_cards || [];
				votingDisabledCards = [];
				beautyDisabledCards = [];
				clearClueRatingResults();
				if (gameMode === 'dixit_plus') {
					void refreshMostBeautifulStats();
				}
			} else if (data.StellaAssociate) {
				syncStageTimer(
					data.StellaAssociate.server_time_ms,
					data.StellaAssociate.current_stage_deadline_s
				);
				setStage('StellaAssociate');
				votingLayoutSeed = null;
				stageImages = data.StellaAssociate.board_cards || [];
				myHandImages = [];
				pinnedCards = [];
				stellaActiveClue = data.StellaAssociate.clue_word || '';
				stellaSelectedCards = data.StellaAssociate.selected_cards || [];
				stellaSelectedCounts = {};
				stellaRevealedCards = [];
				stellaRevealedCardChoosers = {};
				stellaCardPoints = {};
				pointChange = {};
				storytellerLeaderboardPointChange.set({});
				beautyLeaderboardPointChange.set({});
				clearClueRatingResults();
			} else if (data.StellaReveal) {
				syncStageTimer(
					data.StellaReveal.server_time_ms,
					data.StellaReveal.current_stage_deadline_s
				);
				setStage('StellaReveal');
				votingLayoutSeed = null;
				stageImages = data.StellaReveal.board_cards || [];
				myHandImages = [];
				pinnedCards = [];
				stellaActiveClue = data.StellaReveal.clue_word || '';
				stellaSelectedCards = data.StellaReveal.selected_cards || [];
				stellaSelectedCounts = data.StellaReveal.selected_counts || {};
				stellaRevealedCards = data.StellaReveal.revealed_cards || [];
				stellaRevealedCardChoosers = data.StellaReveal.revealed_card_choosers || {};
				stellaCardPoints = data.StellaReveal.card_points || {};
				pointChange = data.StellaReveal.point_change || {};
				storytellerLeaderboardPointChange.set({});
				beautyLeaderboardPointChange.set({});
				clearClueRatingResults();
				activePlayer = data.StellaReveal.scout || activePlayer;
				stellaDarkPlayer = data.StellaReveal.dark_player || stellaDarkPlayer;
			} else if (data.StellaResults) {
				syncStageTimer(
					data.StellaResults.server_time_ms,
					data.StellaResults.current_stage_deadline_s
				);
				setStage('StellaResults');
				votingLayoutSeed = null;
				stageImages = data.StellaResults.board_cards || [];
				myHandImages = [];
				pinnedCards = [];
				stellaActiveClue = data.StellaResults.clue_word || '';
				stellaSelectedCounts = data.StellaResults.selected_counts || {};
				stellaRevealedCards = data.StellaResults.revealed_cards || [];
				stellaRevealedCardChoosers = data.StellaResults.revealed_card_choosers || {};
				stellaCardPoints = data.StellaResults.card_points || {};
				pointChange = data.StellaResults.point_change || {};
				storytellerLeaderboardPointChange.set({});
				beautyLeaderboardPointChange.set({});
				clearClueRatingResults();
				stellaDarkPlayer = data.StellaResults.dark_player || stellaDarkPlayer;
			} else if (data.ErrorMsg) {
				if (!hasReceivedRoomState && usingRoomAuthOverride()) {
					rejoin = false;
					migrationJoinError = data.ErrorMsg;
					gameServer.close();
				}
				toastStore.trigger({
					message: '😭 ' + data.ErrorMsg,
					autohide: true,
					timeout: 2500
				});
			} else if (data.InvalidRoomId) {
				rejoin = false;
				clearStoredAssignedName();
				toastStore.trigger({
					message: '💔 Invalid Room Code!',
					autohide: true,
					timeout: 2500
				});
				console.log('hello');
				goto('/');
			} else if (data.Kicked) {
				rejoin = false;
				clearStoredAssignedName();
				toastStore.trigger({
					message: '👢 ' + (data.Kicked.reason || 'You were removed from the lobby'),
					autohide: true,
					timeout: 2500
				});
				gameServer.close();
				goto('/');
			} else if (data.LeftRoom) {
				rejoin = false;
				clearStoredAssignedName();
				toastStore.trigger({
					message: '👋 ' + (data.LeftRoom.reason || 'You left the game'),
					autohide: true,
					timeout: 2000
				});
				gameServer.close();
				goto('/');
			} else if (data.EndGame) {
				setStage('End');
				clearStageTimer();
				storytellerChosenCard = '';
				clearClueRatingResults();
				if (gameMode === 'dixit_plus') {
					void refreshMostBeautifulStats();
				}
			}
		});

		return () => {
			if (typeof window !== 'undefined') {
				window.removeEventListener('pointerdown', maybeUnlockStageChangeAudio);
				window.removeEventListener('keydown', maybeUnlockStageChangeAudio);
			}
		};
	});
</script>

<div class="w-full">
	{#if stageVisualCueVisible}
		{#key stageVisualCueKey}
			<div
				class={`stage-visual-cue-overlay stage-visual-cue-${activeStageVisualCueTone}`}
				aria-hidden="true"
			></div>
		{/key}
	{/if}
	{#if migrationJoinError !== '' && !hasReceivedRoomState}
		<div class="pt-10">
			<div class="mx-auto max-w-xl px-4">
				<div class="card light space-y-4 p-6 text-center">
					<h1 class="text-2xl font-semibold">Device migration link failed</h1>
					<p class="opacity-80">{migrationJoinError}</p>
					<div class="flex justify-center">
						<button class="btn variant-filled" on:click={() => goto(`/?room=${roomCode}`)}>
							Back to home
						</button>
					</div>
				</div>
			</div>
		</div>
	{:else if stage === 'Joining'}
		<div class="pt-10">
			<Joining
				{name}
				{gameServer}
				{players}
				{roomCode}
				{stage}
				{gameMode}
				{winCondition}
				{storytellerPoolEnabled}
				{storytellerPoolActive}
				{storytellerPoolPlayers}
				{storytellerSuccessPoints}
				{storytellerSuccessPointsMin}
				{storytellerSuccessPointsMax}
				{creator}
				{moderators}
				{cardsPerHand}
				{cardsPerHandMin}
				{cardsPerHandMax}
				{beautyEnabled}
				{beautyVotesPerPlayer}
				{beautyVotesPerPlayerMin}
				{beautyVotesPerPlayerMax}
				{beautyAllowDuplicateVotes}
				{beautyPointsBonus}
				{beautyPointsBonusMin}
				{beautyPointsBonusMax}
				{beautyResultsDisplayMode}
				{stellaBoardSize}
				{stellaBoardSizeMin}
				{stellaBoardSizeMax}
				{stellaSelectionMin}
				{stellaSelectionMax}
				{stellaSelectionCountMin}
				{stellaSelectionCountMax}
				{stellaWordPackWords}
				{stellaQueueDuringAssociation}
				{stellaQueuedRevealMode}
				{stellaScoutTimerEnabled}
				{stellaScoutTimerDurationS}
				{forceStellaScoutTimer}
				roomStateLoaded={hasReceivedRoomState}
			/>
		</div>
	{:else if stage === 'StellaAssociate'}
		<StellaAssociate
			{displayImages}
			{name}
			{creator}
			{moderators}
			{observers}
			{activePlayer}
			{gameServer}
			{players}
			{allowNewPlayersMidgame}
			{storytellerLossComplement}
			{storytellerLossComplementMin}
			{storytellerLossComplementMax}
			{storytellerLossComplementAuto}
			{storytellerPoolEnabled}
			{storytellerPoolActive}
			{storytellerPoolPlayers}
			{storytellerSuccessPoints}
			{storytellerSuccessPointsMin}
			{storytellerSuccessPointsMax}
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
			{showPreviousResultsDuringStorytellerChoosing}
			{randomizeVotingCardOrderPerViewer}
			{cardsPerHand}
			{cardsPerHandMin}
			{cardsPerHandMax}
			{nominationsPerGuesser}
			{nominationsPerGuesserMin}
			{nominationsPerGuesserMax}
			{bonusCorrectGuessOnThresholdCorrectLoss}
			{doubleVoteBonusNormalPoints}
			{doubleVoteBonusTooManyWrongPoints}
			{doubleVoteBonusTooManyWrongFollowsNormal}
			{doubleVoteBonusTooManyCorrectPoints}
			{doubleVoteBonusTooManyCorrectFollowsNormal}
			{doubleVoteBonusPointsMin}
			{doubleVoteBonusPointsMax}
			{bonusThresholdLossTogglesApplyToAllStorytellerLossRounds}
			{showVotingCardNumbers}
			{roundStartDiscardAllUnpinned}
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
			{serverTimeMs}
			{currentStageDeadlineS}
			{votingWrongCardDisableDistribution}
			{stage}
			{pointChange}
			{roundNum}
			{cardsRemaining}
			{deckRefillFlashToken}
			{winCondition}
			{gameMode}
			clueWord={stellaActiveClue}
			selectedCards={stellaSelectedCards}
		/>
	{:else if stage === 'StellaReveal'}
		<StellaReveal
			{displayImages}
			{name}
			{creator}
			{moderators}
			{observers}
			{activePlayer}
			{gameServer}
			{players}
			{allowNewPlayersMidgame}
			{storytellerLossComplement}
			{storytellerLossComplementMin}
			{storytellerLossComplementMax}
			{storytellerLossComplementAuto}
			{storytellerPoolEnabled}
			{storytellerPoolActive}
			{storytellerPoolPlayers}
			{storytellerSuccessPoints}
			{storytellerSuccessPointsMin}
			{storytellerSuccessPointsMax}
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
			{showPreviousResultsDuringStorytellerChoosing}
			{randomizeVotingCardOrderPerViewer}
			{cardsPerHand}
			{cardsPerHandMin}
			{cardsPerHandMax}
			{nominationsPerGuesser}
			{nominationsPerGuesserMin}
			{nominationsPerGuesserMax}
			{bonusCorrectGuessOnThresholdCorrectLoss}
			{doubleVoteBonusNormalPoints}
			{doubleVoteBonusTooManyWrongPoints}
			{doubleVoteBonusTooManyWrongFollowsNormal}
			{doubleVoteBonusTooManyCorrectPoints}
			{doubleVoteBonusTooManyCorrectFollowsNormal}
			{doubleVoteBonusPointsMin}
			{doubleVoteBonusPointsMax}
			{bonusThresholdLossTogglesApplyToAllStorytellerLossRounds}
			{showVotingCardNumbers}
			{roundStartDiscardAllUnpinned}
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
			{serverTimeMs}
			{currentStageDeadlineS}
			{votingWrongCardDisableDistribution}
			{stage}
			{pointChange}
			{roundNum}
			{cardsRemaining}
			{deckRefillFlashToken}
			{winCondition}
			{gameMode}
			clueWord={stellaActiveClue}
			selectedCards={stellaSelectedCards}
			selectedCounts={stellaSelectedCounts}
			revealedCards={stellaRevealedCards}
			revealedCardChoosers={stellaRevealedCardChoosers}
			cardPoints={stellaCardPoints}
			darkPlayer={stellaDarkPlayer}
		/>
	{:else if stage === 'ActiveChooses'}
		<ActiveChooses
			{displayImages}
			{pinnedCards}
			{activePlayer}
			{name}
			{creator}
			{moderators}
			{observers}
			{gameServer}
			{players}
			{allowNewPlayersMidgame}
			{storytellerLossComplement}
			{storytellerLossComplementMin}
			{storytellerLossComplementMax}
			{storytellerLossComplementAuto}
			{storytellerPoolEnabled}
			{storytellerPoolActive}
			{storytellerPoolPlayers}
			{storytellerSuccessPoints}
			{storytellerSuccessPointsMin}
			{storytellerSuccessPointsMax}
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
			{showPreviousResultsDuringStorytellerChoosing}
			{previousDixitResults}
			{cardsPerHand}
			{cardsPerHandMin}
			{cardsPerHandMax}
			{nominationsPerGuesser}
			{nominationsPerGuesserMin}
			{nominationsPerGuesserMax}
			{bonusCorrectGuessOnThresholdCorrectLoss}
			{doubleVoteBonusNormalPoints}
			{doubleVoteBonusTooManyWrongPoints}
			{doubleVoteBonusTooManyWrongFollowsNormal}
			{doubleVoteBonusTooManyCorrectPoints}
			{doubleVoteBonusTooManyCorrectFollowsNormal}
			{doubleVoteBonusPointsMin}
			{doubleVoteBonusPointsMax}
			{bonusThresholdLossTogglesApplyToAllStorytellerLossRounds}
			{showVotingCardNumbers}
			{roundStartDiscardAllUnpinned}
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
			{serverTimeMs}
			{currentStageDeadlineS}
			{votingWrongCardDisableDistribution}
			{stage}
			{pointChange}
			{roundNum}
			{cardsRemaining}
			{deckRefillFlashToken}
			{winCondition}
		/>
	{:else if stage === 'PlayersChoose'}
		<PlayersChoose
			{displayImages}
			{pinnedCards}
			{name}
			{creator}
			{moderators}
			{observers}
			{activePlayer}
			{gameServer}
			{description}
			{previousDixitResults}
			{players}
			{allowNewPlayersMidgame}
			{storytellerLossComplement}
			{storytellerLossComplementMin}
			{storytellerLossComplementMax}
			{storytellerLossComplementAuto}
			{storytellerPoolEnabled}
			{storytellerPoolActive}
			{storytellerPoolPlayers}
			{storytellerSuccessPoints}
			{storytellerSuccessPointsMin}
			{storytellerSuccessPointsMax}
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
			{showPreviousResultsDuringStorytellerChoosing}
			{randomizeVotingCardOrderPerViewer}
			{cardsPerHand}
			{cardsPerHandMin}
			{cardsPerHandMax}
			{nominationsPerGuesser}
			{nominationsPerGuesserMin}
			{nominationsPerGuesserMax}
			{bonusCorrectGuessOnThresholdCorrectLoss}
			{doubleVoteBonusNormalPoints}
			{doubleVoteBonusTooManyWrongPoints}
			{doubleVoteBonusTooManyWrongFollowsNormal}
			{doubleVoteBonusTooManyCorrectPoints}
			{doubleVoteBonusTooManyCorrectFollowsNormal}
			{doubleVoteBonusPointsMin}
			{doubleVoteBonusPointsMax}
			{bonusThresholdLossTogglesApplyToAllStorytellerLossRounds}
			{showVotingCardNumbers}
			{roundStartDiscardAllUnpinned}
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
			{serverTimeMs}
			{currentStageDeadlineS}
			{votingWrongCardDisableDistribution}
			{stage}
			{pointChange}
			{roundNum}
			{cardsRemaining}
			{deckRefillFlashToken}
			{winCondition}
			{roomCode}
			serverSelectedCards={playersChooseSelectedCards}
			chosenCard={storytellerChosenCard}
		/>
	{:else if stage === 'Voting'}
		<Voting
			{displayImages}
			{myHandImages}
			{pinnedCards}
			{activePlayer}
			{name}
			{creator}
			{moderators}
			{observers}
			{gameServer}
			{description}
			{players}
			{allowNewPlayersMidgame}
			{storytellerLossComplement}
			{storytellerLossComplementMin}
			{storytellerLossComplementMax}
			{storytellerLossComplementAuto}
			{storytellerPoolEnabled}
			{storytellerPoolActive}
			{storytellerPoolPlayers}
			{storytellerSuccessPoints}
			{storytellerSuccessPointsMin}
			{storytellerSuccessPointsMax}
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
			{showPreviousResultsDuringStorytellerChoosing}
			{randomizeVotingCardOrderPerViewer}
			{cardsPerHand}
			{cardsPerHandMin}
			{cardsPerHandMax}
			{nominationsPerGuesser}
			{nominationsPerGuesserMin}
			{nominationsPerGuesserMax}
			{bonusCorrectGuessOnThresholdCorrectLoss}
			{doubleVoteBonusNormalPoints}
			{doubleVoteBonusTooManyWrongPoints}
			{doubleVoteBonusTooManyWrongFollowsNormal}
			{doubleVoteBonusTooManyCorrectPoints}
			{doubleVoteBonusTooManyCorrectFollowsNormal}
			{doubleVoteBonusPointsMin}
			{doubleVoteBonusPointsMax}
			{bonusThresholdLossTogglesApplyToAllStorytellerLossRounds}
			{showVotingCardNumbers}
			{roundStartDiscardAllUnpinned}
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
			{serverTimeMs}
			{currentStageDeadlineS}
			{votingWrongCardDisableDistribution}
			{stage}
			{pointChange}
			{roundNum}
			{cardsRemaining}
			{deckRefillFlashToken}
			{winCondition}
			cardNumberLabels={displayImageBadgeLabels}
			disabledCards={votingDisabledCards}
		/>
	{:else if stage === 'BeautyVoting'}
		<BeautyVoting
			{displayImages}
			{myHandImages}
			{pinnedCards}
			{activePlayer}
			{name}
			{creator}
			{moderators}
			{observers}
			{gameServer}
			{description}
			{players}
			{allowNewPlayersMidgame}
			{storytellerLossComplement}
			{storytellerLossComplementMin}
			{storytellerLossComplementMax}
			{storytellerLossComplementAuto}
			{storytellerPoolEnabled}
			{storytellerPoolActive}
			{storytellerPoolPlayers}
			{storytellerSuccessPoints}
			{storytellerSuccessPointsMin}
			{storytellerSuccessPointsMax}
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
			{showPreviousResultsDuringStorytellerChoosing}
			{randomizeVotingCardOrderPerViewer}
			{cardsPerHand}
			{cardsPerHandMin}
			{cardsPerHandMax}
			{nominationsPerGuesser}
			{nominationsPerGuesserMin}
			{nominationsPerGuesserMax}
			{bonusCorrectGuessOnThresholdCorrectLoss}
			{doubleVoteBonusNormalPoints}
			{doubleVoteBonusTooManyWrongPoints}
			{doubleVoteBonusTooManyWrongFollowsNormal}
			{doubleVoteBonusTooManyCorrectPoints}
			{doubleVoteBonusTooManyCorrectFollowsNormal}
			{doubleVoteBonusPointsMin}
			{doubleVoteBonusPointsMax}
			{bonusThresholdLossTogglesApplyToAllStorytellerLossRounds}
			{showVotingCardNumbers}
			{roundStartDiscardAllUnpinned}
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
			{serverTimeMs}
			{currentStageDeadlineS}
			{votingWrongCardDisableDistribution}
			{stage}
			{pointChange}
			{roundNum}
			{cardsRemaining}
			{deckRefillFlashToken}
			{winCondition}
			cardNumberLabels={displayImageBadgeLabels}
			disabledCards={beautyDisabledCards}
		/>
	{:else if stage === 'ClueRating'}
		<ClueRating
			{myHandImages}
			{pinnedCards}
			{name}
			{creator}
			{moderators}
			{observers}
			{activePlayer}
			{gameServer}
			{description}
			{players}
			{allowNewPlayersMidgame}
			{storytellerLossComplement}
			{storytellerLossComplementMin}
			{storytellerLossComplementMax}
			{storytellerLossComplementAuto}
			{storytellerPoolEnabled}
			{storytellerPoolActive}
			{storytellerPoolPlayers}
			{storytellerSuccessPoints}
			{storytellerSuccessPointsMin}
			{storytellerSuccessPointsMax}
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
			{showPreviousResultsDuringStorytellerChoosing}
			{randomizeVotingCardOrderPerViewer}
			{cardsPerHand}
			{cardsPerHandMin}
			{cardsPerHandMax}
			{nominationsPerGuesser}
			{nominationsPerGuesserMin}
			{nominationsPerGuesserMax}
			{bonusCorrectGuessOnThresholdCorrectLoss}
			{doubleVoteBonusNormalPoints}
			{doubleVoteBonusTooManyWrongPoints}
			{doubleVoteBonusTooManyWrongFollowsNormal}
			{doubleVoteBonusTooManyCorrectPoints}
			{doubleVoteBonusTooManyCorrectFollowsNormal}
			{doubleVoteBonusPointsMin}
			{doubleVoteBonusPointsMax}
			{bonusThresholdLossTogglesApplyToAllStorytellerLossRounds}
			{showVotingCardNumbers}
			{roundStartDiscardAllUnpinned}
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
			{serverTimeMs}
			{currentStageDeadlineS}
			{votingWrongCardDisableDistribution}
			{stage}
			{pointChange}
			{roundNum}
			{cardsRemaining}
			{deckRefillFlashToken}
			darkPlayer={stellaDarkPlayer}
			{winCondition}
			{gameMode}
			maxStars={clueRatingStageMaxStars}
		/>
	{:else if stage === 'Results'}
		<Results
			{displayImages}
			{myHandImages}
			{pinnedCards}
			cardNumberLabels={displayImageBadgeLabels}
			{name}
			{creator}
			{moderators}
			{observers}
			{gameServer}
			{playerToCurrentCards}
			{playerToVotes}
			{playerToBeautyVotes}
			{activeCard}
			{activePlayer}
			{players}
			{allowNewPlayersMidgame}
			{storytellerLossComplement}
			{storytellerLossComplementMin}
			{storytellerLossComplementMax}
			{storytellerLossComplementAuto}
			{storytellerPoolEnabled}
			{storytellerPoolActive}
			{storytellerPoolPlayers}
			{storytellerSuccessPoints}
			{storytellerSuccessPointsMin}
			{storytellerSuccessPointsMax}
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
			{showPreviousResultsDuringStorytellerChoosing}
			{randomizeVotingCardOrderPerViewer}
			{cardsPerHand}
			{cardsPerHandMin}
			{cardsPerHandMax}
			{nominationsPerGuesser}
			{nominationsPerGuesserMin}
			{nominationsPerGuesserMax}
			{bonusCorrectGuessOnThresholdCorrectLoss}
			{doubleVoteBonusNormalPoints}
			{doubleVoteBonusTooManyWrongPoints}
			{doubleVoteBonusTooManyWrongFollowsNormal}
			{doubleVoteBonusTooManyCorrectPoints}
			{doubleVoteBonusTooManyCorrectFollowsNormal}
			{doubleVoteBonusPointsMin}
			{doubleVoteBonusPointsMax}
			{bonusThresholdLossTogglesApplyToAllStorytellerLossRounds}
			{showVotingCardNumbers}
			{roundStartDiscardAllUnpinned}
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
			{serverTimeMs}
			{currentStageDeadlineS}
			{votingWrongCardDisableDistribution}
			{stage}
			{pointChange}
			{beautyVoteTotals}
			{beautyWinningCards}
			{roundNum}
			{cardsRemaining}
			{deckRefillFlashToken}
			{winCondition}
		/>
	{:else if stage === 'BeautyResults'}
		<BeautyResults
			{displayImages}
			{myHandImages}
			{pinnedCards}
			cardNumberLabels={displayImageBadgeLabels}
			{name}
			{creator}
			{moderators}
			{observers}
			{activePlayer}
			{gameServer}
			{playerToCurrentCards}
			{playerToBeautyVotes}
			{players}
			{allowNewPlayersMidgame}
			{storytellerLossComplement}
			{storytellerLossComplementMin}
			{storytellerLossComplementMax}
			{storytellerLossComplementAuto}
			{storytellerPoolEnabled}
			{storytellerPoolActive}
			{storytellerPoolPlayers}
			{storytellerSuccessPoints}
			{storytellerSuccessPointsMin}
			{storytellerSuccessPointsMax}
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
			{showPreviousResultsDuringStorytellerChoosing}
			{randomizeVotingCardOrderPerViewer}
			{cardsPerHand}
			{cardsPerHandMin}
			{cardsPerHandMax}
			{nominationsPerGuesser}
			{nominationsPerGuesserMin}
			{nominationsPerGuesserMax}
			{bonusCorrectGuessOnThresholdCorrectLoss}
			{doubleVoteBonusNormalPoints}
			{doubleVoteBonusTooManyWrongPoints}
			{doubleVoteBonusTooManyWrongFollowsNormal}
			{doubleVoteBonusTooManyCorrectPoints}
			{doubleVoteBonusTooManyCorrectFollowsNormal}
			{doubleVoteBonusPointsMin}
			{doubleVoteBonusPointsMax}
			{bonusThresholdLossTogglesApplyToAllStorytellerLossRounds}
			{showVotingCardNumbers}
			{roundStartDiscardAllUnpinned}
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
			{serverTimeMs}
			{currentStageDeadlineS}
			{votingWrongCardDisableDistribution}
			{stage}
			{pointChange}
			{beautyVoteTotals}
			{beautyWinningCards}
			{roundNum}
			{cardsRemaining}
			{deckRefillFlashToken}
			{winCondition}
		/>
	{:else if stage === 'StellaResults'}
		<StellaResults
			{displayImages}
			{name}
			{creator}
			{moderators}
			{observers}
			{activePlayer}
			{gameServer}
			{players}
			{allowNewPlayersMidgame}
			{storytellerLossComplement}
			{storytellerLossComplementMin}
			{storytellerLossComplementMax}
			{storytellerLossComplementAuto}
			{storytellerPoolEnabled}
			{storytellerPoolActive}
			{storytellerPoolPlayers}
			{storytellerSuccessPoints}
			{storytellerSuccessPointsMin}
			{storytellerSuccessPointsMax}
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
			{showPreviousResultsDuringStorytellerChoosing}
			{randomizeVotingCardOrderPerViewer}
			{cardsPerHand}
			{cardsPerHandMin}
			{cardsPerHandMax}
			{nominationsPerGuesser}
			{nominationsPerGuesserMin}
			{nominationsPerGuesserMax}
			{bonusCorrectGuessOnThresholdCorrectLoss}
			{doubleVoteBonusNormalPoints}
			{doubleVoteBonusTooManyWrongPoints}
			{doubleVoteBonusTooManyWrongFollowsNormal}
			{doubleVoteBonusTooManyCorrectPoints}
			{doubleVoteBonusTooManyCorrectFollowsNormal}
			{doubleVoteBonusPointsMin}
			{doubleVoteBonusPointsMax}
			{bonusThresholdLossTogglesApplyToAllStorytellerLossRounds}
			{showVotingCardNumbers}
			{roundStartDiscardAllUnpinned}
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
			{serverTimeMs}
			{currentStageDeadlineS}
			{votingWrongCardDisableDistribution}
			{stage}
			{pointChange}
			{roundNum}
			{cardsRemaining}
			{deckRefillFlashToken}
			{winCondition}
			{gameMode}
			clueWord={stellaActiveClue}
			revealedCardChoosers={stellaRevealedCardChoosers}
			cardPoints={stellaCardPoints}
			darkPlayer={stellaDarkPlayer}
		/>
	{:else if stage === 'Paused'}
		<Paused
			{name}
			{creator}
			{moderators}
			{observers}
			{players}
			{gameServer}
			{allowNewPlayersMidgame}
			{storytellerLossComplement}
			{storytellerLossComplementMin}
			{storytellerLossComplementMax}
			{storytellerLossComplementAuto}
			{storytellerPoolEnabled}
			{storytellerPoolActive}
			{storytellerPoolPlayers}
			{storytellerSuccessPoints}
			{storytellerSuccessPointsMin}
			{storytellerSuccessPointsMax}
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
			{showPreviousResultsDuringStorytellerChoosing}
			{randomizeVotingCardOrderPerViewer}
			{cardsPerHand}
			{cardsPerHandMin}
			{cardsPerHandMax}
			{nominationsPerGuesser}
			{nominationsPerGuesserMin}
			{nominationsPerGuesserMax}
			{bonusCorrectGuessOnThresholdCorrectLoss}
			{doubleVoteBonusNormalPoints}
			{doubleVoteBonusTooManyWrongPoints}
			{doubleVoteBonusTooManyWrongFollowsNormal}
			{doubleVoteBonusTooManyCorrectPoints}
			{doubleVoteBonusTooManyCorrectFollowsNormal}
			{doubleVoteBonusPointsMin}
			{doubleVoteBonusPointsMax}
			{bonusThresholdLossTogglesApplyToAllStorytellerLossRounds}
			{showVotingCardNumbers}
			{roundStartDiscardAllUnpinned}
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
			{serverTimeMs}
			{currentStageDeadlineS}
			{votingWrongCardDisableDistribution}
			{roundNum}
			{cardsRemaining}
			{deckRefillFlashToken}
			{winCondition}
			{gameMode}
			reason={pausedReason}
		/>
	{:else if stage === 'End'}
		<div class="pt-10">
			<End
				{players}
				{observers}
				{gameMode}
				{beautyEnabled}
				{name}
				{roundNum}
				{storytellerPoolActive}
				{storytellerPoolPlayers}
			/>
		</div>
	{/if}
	{#if (stage === 'Joining' || stage === 'End') && !(migrationJoinError !== '' && !hasReceivedRoomState)}
		<div class="mx-auto mt-4 max-w-[680px] px-3 pb-6 lg:px-6">
			<ScoreCheatsheet
				{gameMode}
				{activePlayer}
				{storytellerLossComplement}
				{storytellerSuccessPoints}
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
				{doubleVoteBonusNormalPoints}
				{doubleVoteBonusTooManyWrongPoints}
				{doubleVoteBonusTooManyWrongFollowsNormal}
				{doubleVoteBonusTooManyCorrectPoints}
				{doubleVoteBonusTooManyCorrectFollowsNormal}
				{bonusThresholdLossTogglesApplyToAllStorytellerLossRounds}
			/>
		</div>
	{/if}
</div>

<style>
	.stage-visual-cue-overlay {
		--stage-cue-rgb: 96 165 250;
		position: fixed;
		inset: 0;
		z-index: 40;
		pointer-events: none;
		animation: stage-visual-cue-pulse 1.1s ease-out both;
	}

	.stage-visual-cue-overlay::before,
	.stage-visual-cue-overlay::after {
		content: '';
		position: absolute;
		inset: 0;
	}

	.stage-visual-cue-overlay::before {
		inset: 0 0 auto 0;
		height: 0.4rem;
		background: linear-gradient(
			90deg,
			rgb(var(--stage-cue-rgb) / 0),
			rgb(var(--stage-cue-rgb) / 0.82) 14%,
			rgb(var(--stage-cue-rgb) / 0.96) 50%,
			rgb(var(--stage-cue-rgb) / 0.82) 86%,
			rgb(var(--stage-cue-rgb) / 0)
		);
		box-shadow: 0 0 24px rgb(var(--stage-cue-rgb) / 0.56);
	}

	.stage-visual-cue-overlay::after {
		background: radial-gradient(
				circle at top center,
				rgb(var(--stage-cue-rgb) / 0.18),
				transparent 30%
			),
			linear-gradient(180deg, rgb(var(--stage-cue-rgb) / 0.08), transparent 22%);
		box-shadow:
			inset 0 0 0 1px rgb(var(--stage-cue-rgb) / 0.16),
			inset 0 0 96px rgb(var(--stage-cue-rgb) / 0.08),
			inset 0 24px 72px rgb(var(--stage-cue-rgb) / 0.12);
	}

	.stage-visual-cue-choose {
		--stage-cue-rgb: 96 165 250;
	}

	.stage-visual-cue-vote {
		--stage-cue-rgb: 251 191 36;
	}

	.stage-visual-cue-results {
		--stage-cue-rgb: 74 222 128;
	}

	.stage-visual-cue-paused {
		--stage-cue-rgb: 251 146 60;
	}

	.stage-visual-cue-end {
		--stage-cue-rgb: 196 181 253;
	}

	@keyframes stage-visual-cue-pulse {
		0% {
			opacity: 0;
		}

		12% {
			opacity: 1;
		}

		100% {
			opacity: 0;
		}
	}

	@media (prefers-reduced-motion: reduce) {
		.stage-visual-cue-overlay {
			animation-duration: 0.7s;
		}

		.stage-visual-cue-overlay::after {
			background: linear-gradient(180deg, rgb(var(--stage-cue-rgb) / 0.08), transparent 22%);
			box-shadow:
				inset 0 0 0 2px rgb(var(--stage-cue-rgb) / 0.2),
				inset 0 0 54px rgb(var(--stage-cue-rgb) / 0.08);
		}
	}
</style>
