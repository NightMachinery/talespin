<script lang="ts">
	import { onMount } from 'svelte';
	import { fade } from 'svelte/transition';
	import { goto } from '$app/navigation';
	import { getToastStore } from '@skeletonlabs/skeleton';

	import { nameStore } from '$lib/store';
	import { get } from 'svelte/store';
	import { http_host } from '$lib/gameServer';
	import type { WinCondition } from '$lib/types';

	type LobbyWinMode = WinCondition['mode'];

	let name = get(nameStore) || '';
	let roomCode = '';
	let joinGameClicked = false;
	let lockedRoomCode = false;
	let toastStore = getToastStore();
	let winMode: LobbyWinMode = 'points';
	let targetPoints = 10;
	let targetCycles = 1;

	$: nameStore.set(name);

	onMount(() => {
		const url = new URL(window.location.href);
		const linkedRoomCode = (url.searchParams.get('room') || '').trim().toLowerCase();
		if (linkedRoomCode) {
			roomCode = linkedRoomCode;
			joinGameClicked = true;
			lockedRoomCode = true;
		}
	});

	function normalizedPositiveInt(value: number, fallback: number): number {
		const normalized = Math.floor(value);
		if (!Number.isFinite(normalized) || normalized <= 0) {
			return fallback;
		}
		return normalized;
	}

	function getWinConditionPayload(): WinCondition {
		if (winMode === 'cycles') {
			return {
				mode: 'cycles',
				target_cycles: normalizedPositiveInt(targetCycles, 1)
			};
		}

		if (winMode === 'cards_finish') {
			return { mode: 'cards_finish' };
		}

		return {
			mode: 'points',
			target_points: normalizedPositiveInt(targetPoints, 10)
		};
	}

	async function createGame() {
		if (lockedRoomCode) {
			return joinGame();
		}

		if (roomCode !== '') {
			return joinGame();
		}

		let res = await fetch(`${http_host}/create`, {
			method: 'POST',
			headers: {
				'Content-Type': 'application/json'
			},
			body: JSON.stringify({
				win_condition: getWinConditionPayload()
			})
		});
		res = await res.json();

		if ((<any>res).RoomState) {
			goto(`/game/${(<any>res).RoomState.room_id}`);
		}
	}

	async function joinGame() {
		if (joinGameClicked) {
			let res = await fetch(`${http_host}/exists`, {
				method: 'POST',
				body: JSON.stringify(roomCode),
				headers: {
					'Content-Type': 'application/json'
				}
			});
			res = await res.json();

			if (res) {
				goto(`/game/${roomCode}`);
			} else {
				toastStore.trigger({
					message: '😭 Room does not exist',
					autohide: true,
					timeout: 2500
				});
			}
		} else {
			joinGameClicked = true;
		}
	}
</script>

<div class="max-w-md mx-auto p-4 mt-5">
	<h1 class="h1">
		<span
			class="bg-gradient-to-br from-red-500 to-yellow-500 bg-clip-text text-transparent box-decoration-clone"
			>Play Talespin!</span
		>
	</h1>

	<div class="card p-4 mt-8">
		<div class="mb-4">
			<label for="name">Name:</label>
			<input
				type="text"
				id="name"
				bind:value={name}
				class="shadow appearance-none border rounded w-full py-2 px-3 text-gray-700 leading-tight focus:outline-none focus:shadow-outline"
			/>
		</div>

		{#if joinGameClicked}
			<div transition:fade>
				<label for="roomCode">Room Code:</label>
				<input
					type="text"
					id="roomCode"
					bind:value={roomCode}
					disabled={lockedRoomCode}
					class="shadow appearance-none border rounded w-full py-2 px-3 text-gray-700 mb-4 leading-tight focus:outline-none focus:shadow-outline"
				/>
			</div>
		{/if}

		<div class="mb-4 space-y-3">
			<label for="winMode">Win Condition:</label>
			<select
				id="winMode"
				bind:value={winMode}
				disabled={lockedRoomCode}
				class="shadow appearance-none border rounded w-full py-2 px-3 text-gray-700 leading-tight focus:outline-none focus:shadow-outline"
			>
				<option value="points">Points target</option>
				<option value="cycles">Storyteller cycles</option>
				<option value="cards_finish">Cards finish</option>
			</select>

			{#if winMode === 'points'}
				<div>
					<label for="targetPoints">Winning points:</label>
					<input
						type="number"
						id="targetPoints"
						min="1"
						step="1"
						bind:value={targetPoints}
						disabled={lockedRoomCode}
						class="shadow appearance-none border rounded w-full py-2 px-3 text-gray-700 leading-tight focus:outline-none focus:shadow-outline"
					/>
				</div>
			{:else if winMode === 'cycles'}
				<div>
					<label for="targetCycles">Full storyteller cycles:</label>
					<input
						type="number"
						id="targetCycles"
						min="1"
						step="1"
						bind:value={targetCycles}
						disabled={lockedRoomCode}
						class="shadow appearance-none border rounded w-full py-2 px-3 text-gray-700 leading-tight focus:outline-none focus:shadow-outline"
					/>
				</div>
			{/if}
		</div>

		<div class="flex justify-between mb-4">
			<button disabled={lockedRoomCode} on:click={() => createGame()} class="btn variant-filled">
				Create Game
			</button>
			<button on:click={() => joinGame()} class="btn variant-filled">Join Game</button>
		</div>
	</div>
	<div class="flex justify-center mt-4">
		<a href="/how-to-play" class="link text-center underline">Instructions</a>
	</div>
</div>
