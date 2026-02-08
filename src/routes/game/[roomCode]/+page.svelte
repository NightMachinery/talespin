<script lang="ts">
	import { onMount, onDestroy } from 'svelte';

	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import { getToastStore } from '@skeletonlabs/skeleton';

	import { nameStore } from '$lib/store';
	import type { PlayerInfo, WinCondition } from '$lib/types';
	import GameServer from '$lib/gameServer';

	import Joining from './Joining.svelte';
	import ActiveChooses from './ActiveChooses.svelte';
	import PlayersChoose from './PlayersChoose.svelte';
	import Voting from './Voting.svelte';
	import Results from './Results.svelte';
	import End from './End.svelte';

	// connection information
	let name = '';
	let roomCode = '';
	let gameServer: GameServer;
	let rejoin = false;

	// game state
	let players: { [key: string]: PlayerInfo } = {};
	let stage: string = 'Joining';
	let creator = '';
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
		gameServer.joinRoom(roomCode, name);
		gameServer.onclose(() => {
			if (rejoin) {
				gameServer.joinRoom(roomCode, name);
			}
		});
		gameServer.addMsgHandler((data: any) => {
			console.log(data);

			if (data.RoomState) {
				const previousDeckRefillCount = deckRefillCount;
				players = data.RoomState.players;
				creator = data.RoomState.creator || '';
				stage = data.RoomState.stage;
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
				roomStateLoaded={hasReceivedRoomState}
			/>
		</div>
	{:else if stage === 'ActiveChooses'}
		<ActiveChooses
			{displayImages}
			{activePlayer}
			{name}
			{gameServer}
			{players}
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
			{activePlayer}
			{gameServer}
			{description}
			{players}
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
			{gameServer}
			{description}
			{players}
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
			{gameServer}
			{playerToCurrentCard}
			{playerToVote}
			{activeCard}
			{activePlayer}
			{players}
			{stage}
			{pointChange}
			{roundNum}
			{cardsRemaining}
			{deckRefillFlashToken}
			{winCondition}
		/>
	{:else if stage === 'End'}
		<div class="pt-10">
			<End {players} />
		</div>
	{/if}
</div>
