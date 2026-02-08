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
	export let creator = '';
	export let moderators: string[] = [];
	export let winCondition: WinCondition = {
		mode: 'points',
		target_points: 10
	};
	const toastStore = getToastStore();
	$: playerEntries = Object.entries(players);
	$: moderatorSet = new Set(moderators);
	$: isCreator = creator !== '' && creator === name;
	$: isModerator = moderatorSet.has(name);
	$: canStart = roomStateLoaded && isModerator && playerEntries.length >= 3;

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

	function startGame() {
		gameServer.startGame();
	}

	function isPlayerModerator(playerName: string) {
		return moderatorSet.has(playerName);
	}

	function kickPlayer(playerName: string) {
		if (!isModerator || playerName === creator || playerName === name) return;
		if (!browser || window.confirm(`Kick ${playerName} from this lobby?`)) {
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
		<div class="container mt-10 flex flex-col gap-3">
			{#each playerEntries as [playerName, value]}
				<div class="card p-3">
					<div class="flex items-center justify-between gap-3">
						<div class="flex items-center gap-3">
							<Avatar
								initials={getInitialsFromString(playerName)}
								background={value.ready ? 'bg-success-500' : 'bg-error-500'}
							/>
							<div class="font-bold">
								{playerName}
								{#if playerName === creator}
									<span class="ml-2 text-xs font-normal opacity-70">(creator)</span>
								{:else if isPlayerModerator(playerName)}
									<span class="ml-2 text-xs font-normal opacity-70">(mod)</span>
								{/if}
							</div>
						</div>
							<div class="flex items-center gap-2">
								{#if playerName !== creator}
									{#if isCreator && isPlayerModerator(playerName)}
										<button
											class="btn variant-filled px-3 py-1 text-sm"
											on:click={() => setModerator(playerName, false)}
										>
											Demote
										</button>
									{:else if (isCreator || isModerator) && !isPlayerModerator(playerName)}
										<button
											class="btn variant-filled px-3 py-1 text-sm"
											on:click={() => setModerator(playerName, true)}
										>
										Make Mod
									</button>
								{/if}
							{/if}
							{#if isModerator && playerName !== creator && playerName !== name}
								<button
									class="btn variant-filled px-3 py-1 text-sm"
									on:click={() => kickPlayer(playerName)}
								>
									Kick
								</button>
							{/if}
						</div>
					</div>
				</div>
			{/each}
		</div>

		<div class="flex flex-col gap-2 mt-10">
			{#if isModerator}
				<button class="btn variant-filled" disabled={!canStart} on:click={startGame}>Start Game</button>
				{#if playerEntries.length < 3}
					<p class="text-center text-sm opacity-70">Need at least 3 players to start.</p>
				{/if}
			{:else}
				<p class="text-center text-sm opacity-70">Waiting for a moderator to start the game.</p>
			{/if}
		</div>
	</div>
</div>
