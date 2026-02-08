<script lang="ts">
	import { browser } from '$app/environment';
	import type GameServer from '$lib/gameServer';
	import type { PlayerInfo } from '$lib/types';
	import { cardsFitToHeight } from '$lib/viewOptions';

	export let players: { [key: string]: PlayerInfo } = {};
	export let name = '';
	export let creator = '';
	export let moderators: string[] = [];
	export let stage = '';
	export let gameServer: GameServer;

	$: moderatorSet = new Set(moderators);
	$: sortedPlayerEntries = Object.entries(players).sort(([a], [b]) => a.localeCompare(b));
	$: isCreator = creator !== '' && creator === name;
	$: isModerator = moderatorSet.has(name);
	$: showModeration = stage !== 'End' && (isCreator || isModerator);

	function isPlayerModerator(playerName: string) {
		return moderatorSet.has(playerName);
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
</script>

<div class="card light space-y-3 p-4">
	<h2 class="text-lg font-semibold">Options</h2>
	<label class="flex items-start gap-3">
		<input
			type="checkbox"
			class="mt-1 h-4 w-4 cursor-pointer accent-primary-500"
			bind:checked={$cardsFitToHeight}
		/>
		<span class="block font-medium">Cards fit to height</span>
	</label>
	{#if showModeration}
		<details class="rounded border border-white/20 px-3 py-2">
			<summary class="cursor-pointer text-sm font-semibold">Moderation</summary>
			<div class="mt-3 max-h-[45vh] space-y-2 overflow-y-auto pr-1">
				{#each sortedPlayerEntries as [playerName]}
					<div class="rounded border border-white/20 px-2 py-1.5">
						<div class="flex items-center justify-between gap-2">
							<div class="font-semibold">
								{playerName}
								{#if playerName === creator}
									<span class="ml-1 text-xs font-normal opacity-70">(creator)</span>
								{:else if isPlayerModerator(playerName)}
									<span class="ml-1 text-xs font-normal opacity-70">(mod)</span>
								{/if}
							</div>
							<div class="flex items-center gap-1.5">
								{#if playerName !== creator}
									{#if isCreator && isPlayerModerator(playerName)}
										<button
											class="btn variant-filled px-2 py-0.5 text-xs"
											on:click={() => setModerator(playerName, false)}
										>
											Demote
										</button>
									{:else if (isCreator || isModerator) && !isPlayerModerator(playerName)}
										<button
											class="btn variant-filled px-2 py-0.5 text-xs"
											on:click={() => setModerator(playerName, true)}
										>
											Make Mod
										</button>
									{/if}
								{/if}
								{#if isModerator && playerName !== name}
									<button
										class="btn variant-filled px-2 py-0.5 text-xs"
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
		</details>
	{/if}
</div>
