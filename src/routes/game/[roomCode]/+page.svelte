<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { get } from 'svelte/store';

	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import { getToastStore } from '@skeletonlabs/skeleton';

	import {
		clearAssignedRoomName,
		getJoinNameForRoom,
		nameStore,
		playerTokenStore,
		setAssignedRoomName
	} from '$lib/store';
	import {
		playStageChangeCue,
		type StageCueStage,
		unlockStageChangeAudio
	} from '$lib/stageChangeAudio';
	import type { GameMode, ObserverInfo, PlayerInfo, WinCondition } from '$lib/types';
	import { stageChangeSoundCuesEnabled } from '$lib/viewOptions';
	import GameServer from '$lib/gameServer';
	import { DEFAULT_VOTING_WRONG_CARD_DISABLE_DISTRIBUTION } from '$lib/votingWrongCardDisableDistribution';

	import Joining from './Joining.svelte';
	import ActiveChooses from './ActiveChooses.svelte';
	import PlayersChoose from './PlayersChoose.svelte';
	import Voting from './Voting.svelte';
	import Results from './Results.svelte';
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
	let lastJoinAttemptName = '';

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
	let votesPerGuesser = 1;
	let votesPerGuesserMin = 1;
	let votesPerGuesserMax = 1;
	let cardsPerHand = 12;
	let cardsPerHandMin = 1;
	let cardsPerHandMax = 18;
	let nominationsPerGuesser = 1;
	let nominationsPerGuesserMin = 1;
	let nominationsPerGuesserMax = 1;
	let bonusCorrectGuessOnThresholdCorrectLoss = true;
	let bonusDoubleVoteOnThresholdCorrectLoss = true;
	let showVotingCardNumbers = true;
	let roundStartDiscardCount = 3;
	let hintChoosingTimerEnabled = true;
	let hintChoosingTimerDurationS = 60;
	let cardChoosingTimerEnabled = true;
	let cardChoosingTimerDurationS = 30;
	let votingTimerEnabled = true;
	let votingTimerDurationS = 180;
	let forceCardChoosingTimer = false;
	let forceVotingTimer = false;
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
	let stellaWordPackSize = 0;
	let stellaDarkPlayer = '';
	let activePlayer = '';
	let description = '';
	let roundNum = 0;
	let cardsRemaining = 0;
	let deckRefillCount = 0;
	let deckRefillFlashToken = 0;
	let hasReceivedRoomState = false;
	let winCondition: WinCondition = {
		mode: 'points',
		target_points: 10
	};

	// UI state
	let displayImages: string[] = [];
	let votingDisabledCards: string[] = [];
	let storytellerChosenCard = '';

	// results
	let playerToCurrentCards: { [key: string]: string[] } = {};
	let playerToVotes: { [key: string]: string[] } = {};
	let activeCard = '';
	let pointChange: { [key: string]: number } = {};
	let stellaSelectedCards: string[] = [];
	let stellaSelectedCounts: { [key: string]: number } = {};
	let stellaRevealedCards: string[] = [];
	let stellaRevealedCardChoosers: { [key: string]: string[] } = {};
	let stellaCardPoints: { [key: string]: number } = {};
	const GAMEPLAY_STAGE_CUES = new Set<StageCueStage>([
		'ActiveChooses',
		'PlayersChoose',
		'Voting',
		'Results'
	]);

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
		if (suppressCue || nextStage === previousStage || !get(stageChangeSoundCuesEnabled)) return;
		if (!GAMEPLAY_STAGE_CUES.has(nextStage as StageCueStage)) return;
		void playStageChangeCue(nextStage as StageCueStage);
	}

	function maybeUnlockStageChangeAudio() {
		if (!get(stageChangeSoundCuesEnabled)) return;
		void unlockStageChangeAudio();
	}

	function clearStoredAssignedName() {
		if (roomCode !== '') {
			clearAssignedRoomName(roomCode);
		}
	}

	function persistAssignedName(assignedName: string) {
		name = assignedName;
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
		const joinName = getJoinNameForRoom(roomCode, preferredName, token);
		lastJoinAttemptName = joinName;
		name = joinName;
		gameServer.joinRoom(roomCode, joinName, token, includePassword ? roomPassword : undefined);
	}

	onDestroy(() => {
		if (gameServer) {
			rejoin = false;
			gameServer.close();
		}
	});

	onMount(() => {
		roomCode = $page.params.roomCode;
		if (typeof window !== 'undefined') {
			roomPassword = window.sessionStorage.getItem(`room_password_${roomCode}`) || '';
		}

		name = getJoinNameForRoom(roomCode, preferredName, token);
		if (name === '') {
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
		gameServer.addMsgHandler((data: any) => {
			console.log(data);

			if (data.JoinedAs) {
				const assignedName = (data.JoinedAs.name || '').trim();
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
				if (!hasReceivedRoomState && roomPassword !== '' && typeof window !== 'undefined') {
					window.sessionStorage.removeItem(`room_password_${roomCode}`);
					roomPassword = '';
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
				votesPerGuesser = data.RoomState.votes_per_guesser ?? 1;
				votesPerGuesserMin = data.RoomState.votes_per_guesser_min ?? 1;
				votesPerGuesserMax = data.RoomState.votes_per_guesser_max ?? 1;
				cardsPerHand = data.RoomState.cards_per_hand ?? 12;
				cardsPerHandMin = data.RoomState.cards_per_hand_min ?? 1;
				cardsPerHandMax = data.RoomState.cards_per_hand_max ?? 18;
				nominationsPerGuesser = data.RoomState.nominations_per_guesser ?? 1;
				nominationsPerGuesserMin = data.RoomState.nominations_per_guesser_min ?? 1;
				nominationsPerGuesserMax = data.RoomState.nominations_per_guesser_max ?? 1;
				bonusCorrectGuessOnThresholdCorrectLoss =
					data.RoomState.bonus_correct_guess_on_threshold_correct_loss ?? true;
				bonusDoubleVoteOnThresholdCorrectLoss =
					data.RoomState.bonus_double_vote_on_threshold_correct_loss ?? true;
				showVotingCardNumbers = data.RoomState.show_voting_card_numbers ?? true;
				roundStartDiscardCount = data.RoomState.round_start_discard_count ?? 3;
				hintChoosingTimerEnabled = data.RoomState.hint_choosing_timer_enabled ?? true;
				hintChoosingTimerDurationS = data.RoomState.hint_choosing_timer_duration_s ?? 60;
				cardChoosingTimerEnabled = data.RoomState.card_choosing_timer_enabled ?? true;
				cardChoosingTimerDurationS = data.RoomState.card_choosing_timer_duration_s ?? 30;
				votingTimerEnabled = data.RoomState.voting_timer_enabled ?? true;
				votingTimerDurationS = data.RoomState.voting_timer_duration_s ?? 180;
				forceCardChoosingTimer = data.RoomState.force_card_choosing_timer ?? false;
				forceVotingTimer = data.RoomState.force_voting_timer ?? false;
				serverTimeMs = data.RoomState.server_time_ms ?? null;
				currentStageDeadlineS = data.RoomState.current_stage_deadline_s ?? null;
				votingWrongCardDisableDistribution = data.RoomState
					.voting_wrong_card_disable_distribution ?? [
					...DEFAULT_VOTING_WRONG_CARD_DISABLE_DISTRIBUTION
				];
				stellaBoardSize = data.RoomState.stella_board_size ?? 15;
				stellaBoardSizeMin = data.RoomState.stella_board_size_min ?? 1;
				stellaBoardSizeMax = data.RoomState.stella_board_size_max ?? 100;
				stellaSelectionMin = data.RoomState.stella_selection_min ?? 1;
				stellaSelectionMax = data.RoomState.stella_selection_max ?? 10;
				stellaSelectionCountMin = data.RoomState.stella_selection_count_min ?? 1;
				stellaSelectionCountMax = data.RoomState.stella_selection_count_max ?? 15;
				stellaActiveClue = data.RoomState.stella_active_clue || '';
				stellaWordPackSize = data.RoomState.stella_word_pack_size ?? 0;
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
				if (!rejoin) {
					toastStore.trigger({
						message: '👋 Connected to room!',
						autohide: true,
						timeout: 2500
					});
					rejoin = true;
				}
			} else if (data.StartRound) {
				setStage('ActiveChooses');
				displayImages = data.StartRound.hand;
				storytellerChosenCard = '';
			} else if (data.PlayersChoose) {
				setStage('PlayersChoose');
				displayImages = data.PlayersChoose.hand;
				description = data.PlayersChoose.description;
				storytellerChosenCard = data.PlayersChoose.chosen_card || '';
			} else if (data.BeginVoting) {
				setStage('Voting');
				displayImages = data.BeginVoting.center_cards;
				description = data.BeginVoting.description;
				votingDisabledCards = data.BeginVoting.disabled_cards || [];
				votesPerGuesser = data.BeginVoting.votes_per_guesser ?? votesPerGuesser;
				storytellerChosenCard = '';
			} else if (data.Results) {
				setStage('Results');
				currentStageDeadlineS = null;
				playerToCurrentCards = data.Results.player_to_current_cards || {};
				displayImages = data.Results.center_cards || Object.values(playerToCurrentCards).flat();
				playerToVotes = data.Results.player_to_votes || {};
				activeCard = data.Results.active_card;
				pointChange = data.Results.point_change;
				votingDisabledCards = [];
				storytellerChosenCard = '';
			} else if (data.StellaAssociate) {
				setStage('StellaAssociate');
				displayImages = data.StellaAssociate.board_cards || [];
				stellaActiveClue = data.StellaAssociate.clue_word || '';
				stellaSelectedCards = data.StellaAssociate.selected_cards || [];
				stellaSelectedCounts = {};
				stellaRevealedCards = [];
				stellaRevealedCardChoosers = {};
				stellaCardPoints = {};
				pointChange = {};
			} else if (data.StellaReveal) {
				setStage('StellaReveal');
				displayImages = data.StellaReveal.board_cards || [];
				stellaActiveClue = data.StellaReveal.clue_word || '';
				stellaSelectedCards = data.StellaReveal.selected_cards || [];
				stellaSelectedCounts = data.StellaReveal.selected_counts || {};
				stellaRevealedCards = data.StellaReveal.revealed_cards || [];
				stellaRevealedCardChoosers = data.StellaReveal.revealed_card_choosers || {};
				stellaCardPoints = data.StellaReveal.card_points || {};
				pointChange = data.StellaReveal.point_change || {};
				activePlayer = data.StellaReveal.scout || activePlayer;
				stellaDarkPlayer = data.StellaReveal.dark_player || stellaDarkPlayer;
			} else if (data.StellaResults) {
				setStage('StellaResults');
				displayImages = data.StellaResults.board_cards || [];
				stellaActiveClue = data.StellaResults.clue_word || '';
				stellaSelectedCounts = data.StellaResults.selected_counts || {};
				stellaRevealedCards = data.StellaResults.revealed_cards || [];
				stellaRevealedCardChoosers = data.StellaResults.revealed_card_choosers || {};
				stellaCardPoints = data.StellaResults.card_points || {};
				pointChange = data.StellaResults.point_change || {};
				stellaDarkPlayer = data.StellaResults.dark_player || stellaDarkPlayer;
			} else if (data.ErrorMsg) {
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
				currentStageDeadlineS = null;
				storytellerChosenCard = '';
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
	{#if stage === 'Joining'}
		<div class="pt-10">
			<Joining
				{name}
				{gameServer}
				{players}
				{roomCode}
				{stage}
				{gameMode}
				{winCondition}
				{creator}
				{moderators}
				{cardsPerHand}
				{cardsPerHandMin}
				{cardsPerHandMax}
				{stellaBoardSize}
				{stellaBoardSizeMin}
				{stellaBoardSizeMax}
				{stellaSelectionMin}
				{stellaSelectionMax}
				{stellaSelectionCountMin}
				{stellaSelectionCountMax}
				{stellaWordPackSize}
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
			{votesPerGuesser}
			{votesPerGuesserMin}
			{votesPerGuesserMax}
			{cardsPerHand}
			{cardsPerHandMin}
			{cardsPerHandMax}
			{nominationsPerGuesser}
			{nominationsPerGuesserMin}
			{nominationsPerGuesserMax}
			{bonusCorrectGuessOnThresholdCorrectLoss}
			{bonusDoubleVoteOnThresholdCorrectLoss}
			{showVotingCardNumbers}
			{roundStartDiscardCount}
			{hintChoosingTimerEnabled}
			{hintChoosingTimerDurationS}
			{cardChoosingTimerEnabled}
			{cardChoosingTimerDurationS}
			{votingTimerEnabled}
			{votingTimerDurationS}
			{forceCardChoosingTimer}
			{forceVotingTimer}
			{stellaBoardSize}
			{stellaBoardSizeMin}
			{stellaBoardSizeMax}
			{stellaSelectionMin}
			{stellaSelectionMax}
			{stellaSelectionCountMin}
			{stellaSelectionCountMax}
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
			{votesPerGuesser}
			{votesPerGuesserMin}
			{votesPerGuesserMax}
			{cardsPerHand}
			{cardsPerHandMin}
			{cardsPerHandMax}
			{nominationsPerGuesser}
			{nominationsPerGuesserMin}
			{nominationsPerGuesserMax}
			{bonusCorrectGuessOnThresholdCorrectLoss}
			{bonusDoubleVoteOnThresholdCorrectLoss}
			{showVotingCardNumbers}
			{roundStartDiscardCount}
			{hintChoosingTimerEnabled}
			{hintChoosingTimerDurationS}
			{cardChoosingTimerEnabled}
			{cardChoosingTimerDurationS}
			{votingTimerEnabled}
			{votingTimerDurationS}
			{forceCardChoosingTimer}
			{forceVotingTimer}
			{stellaBoardSize}
			{stellaBoardSizeMin}
			{stellaBoardSizeMax}
			{stellaSelectionMin}
			{stellaSelectionMax}
			{stellaSelectionCountMin}
			{stellaSelectionCountMax}
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
			{votesPerGuesser}
			{votesPerGuesserMin}
			{votesPerGuesserMax}
			{cardsPerHand}
			{cardsPerHandMin}
			{cardsPerHandMax}
			{nominationsPerGuesser}
			{nominationsPerGuesserMin}
			{nominationsPerGuesserMax}
			{bonusCorrectGuessOnThresholdCorrectLoss}
			{bonusDoubleVoteOnThresholdCorrectLoss}
			{showVotingCardNumbers}
			{roundStartDiscardCount}
			{hintChoosingTimerEnabled}
			{hintChoosingTimerDurationS}
			{cardChoosingTimerEnabled}
			{cardChoosingTimerDurationS}
			{votingTimerEnabled}
			{votingTimerDurationS}
			{forceCardChoosingTimer}
			{forceVotingTimer}
			{stellaBoardSize}
			{stellaBoardSizeMin}
			{stellaBoardSizeMax}
			{stellaSelectionMin}
			{stellaSelectionMax}
			{stellaSelectionCountMin}
			{stellaSelectionCountMax}
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
			{votesPerGuesser}
			{votesPerGuesserMin}
			{votesPerGuesserMax}
			{cardsPerHand}
			{cardsPerHandMin}
			{cardsPerHandMax}
			{nominationsPerGuesser}
			{nominationsPerGuesserMin}
			{nominationsPerGuesserMax}
			{bonusCorrectGuessOnThresholdCorrectLoss}
			{bonusDoubleVoteOnThresholdCorrectLoss}
			{showVotingCardNumbers}
			{roundStartDiscardCount}
			{hintChoosingTimerEnabled}
			{hintChoosingTimerDurationS}
			{cardChoosingTimerEnabled}
			{cardChoosingTimerDurationS}
			{votingTimerEnabled}
			{votingTimerDurationS}
			{forceCardChoosingTimer}
			{forceVotingTimer}
			{stellaBoardSize}
			{stellaBoardSizeMin}
			{stellaBoardSizeMax}
			{stellaSelectionMin}
			{stellaSelectionMax}
			{stellaSelectionCountMin}
			{stellaSelectionCountMax}
			{serverTimeMs}
			{currentStageDeadlineS}
			{votingWrongCardDisableDistribution}
			{stage}
			{pointChange}
			{roundNum}
			{cardsRemaining}
			{deckRefillFlashToken}
			{winCondition}
			chosenCard={storytellerChosenCard}
		/>
	{:else if stage === 'Voting'}
		<Voting
			{displayImages}
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
			{votesPerGuesser}
			{votesPerGuesserMin}
			{votesPerGuesserMax}
			{cardsPerHand}
			{cardsPerHandMin}
			{cardsPerHandMax}
			{nominationsPerGuesser}
			{nominationsPerGuesserMin}
			{nominationsPerGuesserMax}
			{bonusCorrectGuessOnThresholdCorrectLoss}
			{bonusDoubleVoteOnThresholdCorrectLoss}
			{showVotingCardNumbers}
			{roundStartDiscardCount}
			{hintChoosingTimerEnabled}
			{hintChoosingTimerDurationS}
			{cardChoosingTimerEnabled}
			{cardChoosingTimerDurationS}
			{votingTimerEnabled}
			{votingTimerDurationS}
			{forceCardChoosingTimer}
			{forceVotingTimer}
			{stellaBoardSize}
			{stellaBoardSizeMin}
			{stellaBoardSizeMax}
			{stellaSelectionMin}
			{stellaSelectionMax}
			{stellaSelectionCountMin}
			{stellaSelectionCountMax}
			{serverTimeMs}
			{currentStageDeadlineS}
			{votingWrongCardDisableDistribution}
			{stage}
			{pointChange}
			{roundNum}
			{cardsRemaining}
			{deckRefillFlashToken}
			{winCondition}
			disabledCards={votingDisabledCards}
		/>
	{:else if stage === 'Results'}
		<Results
			{displayImages}
			{name}
			{creator}
			{moderators}
			{observers}
			{gameServer}
			{playerToCurrentCards}
			{playerToVotes}
			{activeCard}
			{activePlayer}
			{players}
			{allowNewPlayersMidgame}
			{storytellerLossComplement}
			{storytellerLossComplementMin}
			{storytellerLossComplementMax}
			{storytellerLossComplementAuto}
			{votesPerGuesser}
			{votesPerGuesserMin}
			{votesPerGuesserMax}
			{cardsPerHand}
			{cardsPerHandMin}
			{cardsPerHandMax}
			{nominationsPerGuesser}
			{nominationsPerGuesserMin}
			{nominationsPerGuesserMax}
			{bonusCorrectGuessOnThresholdCorrectLoss}
			{bonusDoubleVoteOnThresholdCorrectLoss}
			{showVotingCardNumbers}
			{roundStartDiscardCount}
			{hintChoosingTimerEnabled}
			{hintChoosingTimerDurationS}
			{cardChoosingTimerEnabled}
			{cardChoosingTimerDurationS}
			{votingTimerEnabled}
			{votingTimerDurationS}
			{forceCardChoosingTimer}
			{forceVotingTimer}
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
			{votesPerGuesser}
			{votesPerGuesserMin}
			{votesPerGuesserMax}
			{cardsPerHand}
			{cardsPerHandMin}
			{cardsPerHandMax}
			{nominationsPerGuesser}
			{nominationsPerGuesserMin}
			{nominationsPerGuesserMax}
			{bonusCorrectGuessOnThresholdCorrectLoss}
			{bonusDoubleVoteOnThresholdCorrectLoss}
			{showVotingCardNumbers}
			{roundStartDiscardCount}
			{hintChoosingTimerEnabled}
			{hintChoosingTimerDurationS}
			{cardChoosingTimerEnabled}
			{cardChoosingTimerDurationS}
			{votingTimerEnabled}
			{votingTimerDurationS}
			{forceCardChoosingTimer}
			{forceVotingTimer}
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
			{votesPerGuesser}
			{votesPerGuesserMin}
			{votesPerGuesserMax}
			{cardsPerHand}
			{cardsPerHandMin}
			{cardsPerHandMax}
			{nominationsPerGuesser}
			{nominationsPerGuesserMin}
			{nominationsPerGuesserMax}
			{bonusCorrectGuessOnThresholdCorrectLoss}
			{bonusDoubleVoteOnThresholdCorrectLoss}
			{showVotingCardNumbers}
			{roundStartDiscardCount}
			{hintChoosingTimerEnabled}
			{hintChoosingTimerDurationS}
			{cardChoosingTimerEnabled}
			{cardChoosingTimerDurationS}
			{votingTimerEnabled}
			{votingTimerDurationS}
			{forceCardChoosingTimer}
			{forceVotingTimer}
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
			<End {players} />
		</div>
	{/if}
	{#if stage === 'Joining' || stage === 'End'}
		<div class="mx-auto mt-4 max-w-[680px] px-3 pb-6 lg:px-6">
			<ScoreCheatsheet
				{gameMode}
				{activePlayer}
				{votesPerGuesser}
				{votesPerGuesserMax}
				{bonusCorrectGuessOnThresholdCorrectLoss}
				{bonusDoubleVoteOnThresholdCorrectLoss}
			/>
		</div>
	{/if}
</div>
