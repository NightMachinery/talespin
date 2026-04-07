<script lang="ts">
	import { onMount } from 'svelte';
	import { fade } from 'svelte/transition';
	import { goto } from '$app/navigation';
	import { getToastStore } from '@skeletonlabs/skeleton';

	import { nameStore } from '$lib/store';
	import { get } from 'svelte/store';
	import { http_host } from '$lib/gameServer';

	let name = get(nameStore) || '';
	let roomCode = '';
	let roomPassword = '';
	let joinGameClicked = false;
	let lockedRoomCode = false;
	let toastStore = getToastStore();

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

	async function createGame() {
		if (lockedRoomCode) {
			return joinGame();
		}

		if (roomCode !== '') {
			return joinGame();
		}

		const trimmedPassword = roomPassword.trim();
		let res = await fetch(`${http_host}/create`, {
			method: 'POST',
			headers: {
				'Content-Type': 'application/json'
			},
			body: JSON.stringify({
				creator_name: name.trim(),
				...(trimmedPassword !== '' ? { password: trimmedPassword } : {})
			})
		});
		res = await res.json();

		if ((<any>res).RoomState) {
			const createdRoomId = (<any>res).RoomState.room_id;
			if (trimmedPassword !== '' && typeof window !== 'undefined') {
				window.sessionStorage.setItem(`room_password_${createdRoomId}`, trimmedPassword);
			}
			goto(`/game/${createdRoomId}`);
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
				const trimmedPassword = roomPassword.trim();
				if (typeof window !== 'undefined') {
					if (trimmedPassword !== '') {
						window.sessionStorage.setItem(`room_password_${roomCode}`, trimmedPassword);
					} else {
						window.sessionStorage.removeItem(`room_password_${roomCode}`);
					}
				}
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

		<div class="mb-4">
			<label for="roomPassword">Room Password (optional):</label>
			<input
				type="password"
				id="roomPassword"
				bind:value={roomPassword}
				class="shadow appearance-none border rounded w-full py-2 px-3 text-gray-700 leading-tight focus:outline-none focus:shadow-outline"
			/>
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
