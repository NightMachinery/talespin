<script lang="ts">
	import { onMount, onDestroy } from 'svelte';

	import { browser } from '$app/environment';
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
	let votingDisabledCard = '';

	// results
	let playerToCurrentCard: { [key: string]: string } = {};
	let playerToVote: { [key: string]: string } = {};
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

	function leaveGame() {
		if (browser && !window.confirm('Leave this game?')) {
			return;
		}

		if (!gameServer) {
			goto('/');
			return;
		}

		rejoin = false;
		gameServer.leaveRoom();
		gameServer.close();
		goto('/');
	}

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
				votingDisabledCard = data.BeginVoting.disabled_card || '';
			} else if (data.Results) {
				stage = 'Results';
				displayImages = Object.values(data.Results.player_to_current_card);
				playerToCurrentCard = data.Results.player_to_current_card;
				playerToVote = data.Results.player_to_vote;
				activeCard = data.Results.active_card;
				pointChange = data.Results.point_change;
				votingDisabledCard = '';
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
	{#if stage !== 'End'}
		<div class="fixed right-3 top-3 z-40">
			<button class="btn variant-filled px-3 py-1 text-sm" on:click={leaveGame}>Leave</button>
		</div>
	{/if}

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
			{stage}
			{pointChange}
			{roundNum}
			{cardsRemaining}
			{deckRefillFlashToken}
			{winCondition}
			disabledCard={votingDisabledCard}
		/>
	{:else if stage === 'Results'}
		<Results
			{displayImages}
			{name}
			{creator}
			{moderators}
			{observers}
			{gameServer}
			{playerToCurrentCard}
			{playerToVote}
			{activeCard}
			{activePlayer}
			{players}
			{allowNewPlayersMidgame}
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
			reason={pausedReason}
		/>
	{:else if stage === 'End'}
		<div class="pt-10">
			<End {players} />
		</div>
	{/if}
</div>
