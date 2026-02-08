<script lang="ts">
	import { browser } from '$app/environment';
	import type GameServer from '$lib/gameServer';
	import type { PlayerInfo, WinCondition } from '$lib/types';
	import { Avatar, getToastStore } from '@skeletonlabs/skeleton';

	export let players: { [key: string]: PlayerInfo } = {};
	export let roomCode: string = '';
	export let gameServer: GameServer;
	export let name = '';
	export let roomStateLoaded = false;
	export let winCondition: WinCondition = {
		mode: 'points',
		target_points: 10
	};
	const toastStore = getToastStore();

	function getInitialsFromString(name: string) {
		return name
			.split(' ')
			.map((n) => n[0])
			.join('');
	}

	function getInviteLink() {
		if (!browser) return '';
		const inviteUrl = new URL('/', window.location.origin);
		inviteUrl.searchParams.set('room', roomCode);
		inviteUrl.searchParams.set('win_mode', winCondition.mode);

		if (winCondition.mode === 'points') {
			inviteUrl.searchParams.set('target_points', String(winCondition.target_points));
		} else if (winCondition.mode === 'cycles') {
			inviteUrl.searchParams.set('target_cycles', String(winCondition.target_cycles));
		}

		return inviteUrl.toString();
	}

	async function copyInviteLink() {
		const inviteLink = getInviteLink();
		if (!inviteLink) return;

		try {
			await navigator.clipboard.writeText(inviteLink);
		} catch {
			const textArea = document.createElement('textarea');
			textArea.value = inviteLink;
			textArea.style.position = 'fixed';
			textArea.style.opacity = '0';
			document.body.appendChild(textArea);
			textArea.focus();
			textArea.select();
			document.execCommand('copy');
			document.body.removeChild(textArea);
		}

		toastStore.trigger({
			message: '🔗 Invite link copied',
			autohide: true,
			timeout: 2000
		});
	}
</script>

<div class="m-auto w-80/10">
	<div class="max-w-md mx-auto">
		<h1 class="text-3xl text-center">Hi {name}, let's play Talespin!</h1>
		<h2 class="text-xl text-center">
			You are in room
			<code class="code text-lg">{roomCode}</code>
		</h2>
		<div class="flex justify-center mt-4">
			<button
				class="btn variant-filled"
				disabled={!roomStateLoaded}
				on:click={() => copyInviteLink()}
			>
				Copy Invite Link
			</button>
		</div>
		<div class="container flex flex-wrap justify-center gap-4 mt-10">
			{#each Object.entries(players) as [key, value]}
				<div class=" p-5">
					<div>
						<Avatar
							initials={getInitialsFromString(key)}
							background={value.ready ? 'bg-success-500' : 'bg-error-500'}
						/>
					</div>

					<div class="font-bold text-center">{key}</div>
				</div>
			{/each}
		</div>

		<div class="flex flex-col gap-2 mt-10">
			<button
				disabled={players && players[name] ? players[name].ready : false}
				class="btn variant-filled"
				on:click={() => gameServer.ready()}>Ready</button
			>
		</div>
	</div>
</div>
