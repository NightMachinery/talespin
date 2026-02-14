<script lang="ts">
	import { onMount, onDestroy } from 'svelte';

	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import { getToastStore } from '@skeletonlabs/skeleton';

	import { nameStore, playerTokenStore } from '$lib/store';
	import type { ObserverInfo, PlayerInfo, WinCondition } from '$lib/types';
	import GameServer from '$lib/gameServer';

	import Joining from './Joining.svelte';
	import ActiveChooses from './ActiveChooses.svelte';
	import PlayersChoose from './PlayersChoose.svelte';
	import Voting from './Voting.svelte';
	import Results from './Results.svelte';
	import Paused from './Paused.svelte';
	import End from './End.svelte';
	import ScoreCheatsheet from './ScoreCheatsheet.svelte';

	// connection information
	let name = '';
	let token = '';
	let roomCode = '';
	let gameServer: GameServer;
	let rejoin = false;

	// game state
	let players: { [key: string]: PlayerInfo } = {};
	let observers: { [key: string]: ObserverInfo } = {};
	let stage: string = 'Joining';
	let creator = '';
	let moderators: string[] = [];
	let allowNewPlayersMidgame = true;
	let pausedReason = '';
	let storytellerLossComplement = 0;
	let storytellerLossComplementMin = 0;
	let storytellerLossComplementMax = 0;
	let votesPerGuesser = 1;
	let votesPerGuesserMin = 1;
	let votesPerGuesserMax = 1;
	let cardsPerHand = 6;
	let cardsPerHandMin = 1;
	let cardsPerHandMax = 12;
	let nominationsPerGuesser = 1;
	let nominationsPerGuesserMin = 1;
	let nominationsPerGuesserMax = 1;
	let bonusCorrectGuessOnThresholdCorrectLoss = false;
	let bonusDoubleVoteOnThresholdCorrectLoss = false;
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

	// results
	let playerToCurrentCards: { [key: string]: string[] } = {};
	let playerToVotes: { [key: string]: string[] } = {};
	let activeCard = '';
	let pointChange: { [key: string]: number } = {};

	// store
	let toastStore = getToastStore();
	nameStore.subscribe((value) => (name = value));
	playerTokenStore.subscribe((value) => (token = value));

	onDestroy(() => {
		if (gameServer) {
			rejoin = false;
			gameServer.close();
		}
	});

	onMount(() => {
		roomCode = $page.params.roomCode;

		if (name === '') {
			goto(`/?room=${roomCode}`);
			return;
		}

		gameServer = new GameServer();
		gameServer.joinRoom(roomCode, name, token);
		gameServer.onclose(() => {
			if (rejoin) {
				gameServer.joinRoom(roomCode, name, token);
			}
		});
		gameServer.addMsgHandler((data: any) => {
			console.log(data);

			if (data.RoomState) {
				const previousDeckRefillCount = deckRefillCount;
				players = data.RoomState.players;
				observers = data.RoomState.observers || {};
				creator = data.RoomState.creator || '';
				moderators = data.RoomState.moderators || [];
				stage = data.RoomState.stage;
				allowNewPlayersMidgame = data.RoomState.allow_new_players_midgame ?? true;
				pausedReason = data.RoomState.paused_reason || '';
				storytellerLossComplement = data.RoomState.storyteller_loss_complement ?? 0;
				storytellerLossComplementMin = data.RoomState.storyteller_loss_complement_min ?? 0;
				storytellerLossComplementMax = data.RoomState.storyteller_loss_complement_max ?? 0;
				votesPerGuesser = data.RoomState.votes_per_guesser ?? 1;
				votesPerGuesserMin = data.RoomState.votes_per_guesser_min ?? 1;
				votesPerGuesserMax = data.RoomState.votes_per_guesser_max ?? 1;
				cardsPerHand = data.RoomState.cards_per_hand ?? 6;
				cardsPerHandMin = data.RoomState.cards_per_hand_min ?? 1;
				cardsPerHandMax = data.RoomState.cards_per_hand_max ?? 12;
				nominationsPerGuesser = data.RoomState.nominations_per_guesser ?? 1;
				nominationsPerGuesserMin = data.RoomState.nominations_per_guesser_min ?? 1;
				nominationsPerGuesserMax = data.RoomState.nominations_per_guesser_max ?? 1;
				bonusCorrectGuessOnThresholdCorrectLoss =
					data.RoomState.bonus_correct_guess_on_threshold_correct_loss ?? false;
				bonusDoubleVoteOnThresholdCorrectLoss =
					data.RoomState.bonus_double_vote_on_threshold_correct_loss ?? false;
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
				stage = 'ActiveChooses';
				displayImages = data.StartRound.hand;
			} else if (data.PlayersChoose) {
				stage = 'PlayersChoose';
				displayImages = data.PlayersChoose.hand;
				description = data.PlayersChoose.description;
			} else if (data.BeginVoting) {
				stage = 'Voting';
				displayImages = data.BeginVoting.center_cards;
				description = data.BeginVoting.description;
				votingDisabledCards = data.BeginVoting.disabled_cards || [];
				votesPerGuesser = data.BeginVoting.votes_per_guesser ?? votesPerGuesser;
			} else if (data.Results) {
				stage = 'Results';
				playerToCurrentCards = data.Results.player_to_current_cards || {};
				displayImages = Object.values(playerToCurrentCards).flat();
				playerToVotes = data.Results.player_to_votes || {};
				activeCard = data.Results.active_card;
				pointChange = data.Results.point_change;
				votingDisabledCards = [];
			} else if (data.ErrorMsg) {
				toastStore.trigger({
					message: '😭 ' + data.ErrorMsg,
					autohide: true,
					timeout: 2500
				});
			} else if (data.InvalidRoomId) {
				rejoin = false;
				toastStore.trigger({
					message: '💔 Invalid Room Code!',
					autohide: true,
					timeout: 2500
				});
				console.log('hello');
				goto('/');
			} else if (data.Kicked) {
				rejoin = false;
				toastStore.trigger({
					message: '👢 ' + (data.Kicked.reason || 'You were removed from the lobby'),
					autohide: true,
					timeout: 2500
				});
				gameServer.close();
				goto('/');
			} else if (data.LeftRoom) {
				rejoin = false;
				toastStore.trigger({
					message: '👋 ' + (data.LeftRoom.reason || 'You left the game'),
					autohide: true,
					timeout: 2000
				});
				gameServer.close();
				goto('/');
			} else if (data.EndGame) {
				stage = 'End';
			}
		});
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
				{winCondition}
				{creator}
				{moderators}
				roomStateLoaded={hasReceivedRoomState}
			/>
		</div>
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
			{stage}
			{pointChange}
			{roundNum}
			{cardsRemaining}
			{deckRefillFlashToken}
			{winCondition}
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
			{stage}
			{pointChange}
			{roundNum}
			{cardsRemaining}
			{deckRefillFlashToken}
			{winCondition}
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
			{roundNum}
			{cardsRemaining}
			{deckRefillFlashToken}
			{winCondition}
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
				{activePlayer}
				{votesPerGuesser}
				{votesPerGuesserMax}
				{bonusCorrectGuessOnThresholdCorrectLoss}
				{bonusDoubleVoteOnThresholdCorrectLoss}
			/>
		</div>
	{/if}
</div>
