<script lang="ts">
	import { browser } from '$app/environment';
	import type GameServer from '$lib/gameServer';
	import type { ObserverInfo, PlayerInfo } from '$lib/types';
	import { OFFLINE_STATUS_LABEL } from '$lib/presence';
	import { cardsFitToHeight } from '$lib/viewOptions';

	export let players: { [key: string]: PlayerInfo } = {};
	export let observers: { [key: string]: ObserverInfo } = {};
	export let name = '';
	export let creator = '';
	export let moderators: string[] = [];
	export let stage = '';
	export let activePlayer = '';
	export let allowNewPlayersMidgame = true;
	export let storytellerLossComplement = 0;
	export let storytellerLossComplementMin = 0;
	export let storytellerLossComplementMax = 0;
	export let votesPerGuesser = 1;
	export let votesPerGuesserMin = 1;
	export let votesPerGuesserMax = 1;
	export let cardsPerHand = 6;
	export let cardsPerHandMin = 1;
	export let cardsPerHandMax = 12;
	export let nominationsPerGuesser = 1;
	export let nominationsPerGuesserMin = 1;
	export let nominationsPerGuesserMax = 1;
	export let gameServer: GameServer;

	$: moderatorSet = new Set(moderators);
	$: sortedPlayerEntries = Object.entries(players).sort(([a], [b]) => a.localeCompare(b));
	$: sortedObserverEntries = Object.entries(observers).sort(([a], [b]) => a.localeCompare(b));
	$: isCreator = creator !== '' && creator === name;
	$: isModerator = moderatorSet.has(name);
	$: showModeration = stage !== 'End' && (isCreator || isModerator);
	$: isSelfObserver = !!observers[name];
	$: selfObserverInfo = observers[name];
	$: selfObserveBlocked = (stage === 'PlayersChoose' || stage === 'Voting') && activePlayer === name;
	$: canBecomeObserver =
		!isSelfObserver && stage !== 'Joining' && stage !== 'End' && !selfObserveBlocked;
	$: canChangeCardsPerHand = stage === 'ActiveChooses';
	$: canChangePreVotingSettings = stage === 'ActiveChooses';
	$: selfJoinPending =
		!!selfObserverInfo && (selfObserverInfo.join_requested || selfObserverInfo.auto_join_on_next_round);
	$: selfJoinBackLabel =
		stage === 'Voting'
			? selfJoinPending
				? 'Cancel pending join'
				: 'Join next round'
			: 'Join now';

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

	function setObserver(playerName: string, enabled: boolean) {
		gameServer.setObserver(playerName, enabled);
	}

	function becomeObserver() {
		if (!canBecomeObserver) return;
		if (!browser || window.confirm('Switch to observer mode?')) {
			gameServer.setObserver(name, true);
		}
	}

	function joinBack() {
		gameServer.requestJoinFromObserver();
	}

	function observerJoinActionLabel(observerInfo: ObserverInfo) {
		if (stage !== 'Voting') return 'Join now';
		const pending = observerInfo.join_requested || observerInfo.auto_join_on_next_round;
		return pending ? 'Cancel pending join' : 'Join next round';
	}

	function updateAllowMidgameJoin(event: Event) {
		const input = event.currentTarget as HTMLInputElement;
		gameServer.setAllowMidgameJoin(input.checked);
	}

	function updateStorytellerLossComplement(event: Event) {
		const input = event.currentTarget as HTMLInputElement;
		const parsed = Number(input.value);
		if (
			!Number.isInteger(parsed) ||
			parsed < storytellerLossComplementMin ||
			parsed > storytellerLossComplementMax
		) {
			input.value = `${storytellerLossComplement}`;
			return;
		}
		if (parsed !== storytellerLossComplement) {
			gameServer.setStorytellerLossComplement(parsed);
		}
	}

	function updateVotesPerGuesser(event: Event) {
		const input = event.currentTarget as HTMLInputElement;
		const parsed = Number(input.value);
		if (!Number.isInteger(parsed) || parsed < votesPerGuesserMin || parsed > votesPerGuesserMax) {
			input.value = `${votesPerGuesser}`;
			return;
		}
		if (parsed !== votesPerGuesser) {
			gameServer.setVotesPerGuesser(parsed);
		}
	}

	function updateCardsPerHand(event: Event) {
		const input = event.currentTarget as HTMLInputElement;
		const parsed = Number(input.value);
		if (!Number.isInteger(parsed) || parsed < cardsPerHandMin || parsed > cardsPerHandMax) {
			input.value = `${cardsPerHand}`;
			return;
		}
		if (parsed !== cardsPerHand) {
			gameServer.setCardsPerHand(parsed);
		}
	}

	function updateNominationsPerGuesser(event: Event) {
		const input = event.currentTarget as HTMLInputElement;
		const parsed = Number(input.value);
		if (
			!Number.isInteger(parsed) ||
			parsed < nominationsPerGuesserMin ||
			parsed > nominationsPerGuesserMax
		) {
			input.value = `${nominationsPerGuesser}`;
			return;
		}
		if (parsed !== nominationsPerGuesser) {
			gameServer.setNominationsPerGuesser(parsed);
		}
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
	{#if stage !== 'Joining' && stage !== 'End'}
		{#if isSelfObserver}
			<button class="btn variant-filled w-full" on:click={joinBack}>{selfJoinBackLabel}</button>
		{:else}
			<button
				class="btn variant-filled w-full"
				on:click={becomeObserver}
				disabled={!canBecomeObserver}
			>
				Become observer
			</button>
			{#if !canBecomeObserver && selfObserveBlocked}
				<p class="text-xs opacity-70">
					Storyteller can only become observer before choosing card and clue.
				</p>
			{/if}
		{/if}
	{/if}
	{#if showModeration}
		<details class="rounded border border-white/20 px-3 py-2">
			<summary class="cursor-pointer text-sm font-semibold">Moderation</summary>
			<div class="mt-3 max-h-[45vh] space-y-2 overflow-y-auto pr-1">
				<p class="text-xs font-semibold uppercase tracking-wide opacity-70">Manage players</p>
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
								{#if isModerator && playerName !== name}
									<button
										class="btn variant-filled px-2 py-0.5 text-xs"
										on:click={() => kickPlayer(playerName)}
									>
										Kick
									</button>
								{/if}
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
								{#if isModerator || playerName === name}
									<button
										class="btn variant-filled px-2 py-0.5 text-xs"
										on:click={() => setObserver(playerName, true)}
									>
										Observer
									</button>
								{/if}
							</div>
						</div>
					</div>
				{/each}
				{#each sortedObserverEntries as [observerName, info]}
					<div class="rounded border border-white/20 px-2 py-1.5 opacity-85">
						<div class="flex items-center justify-between gap-2">
							<div class="font-semibold">
								{observerName}
								<span class="ml-1 text-xs font-normal opacity-70">(observer)</span>
								{#if !info.connected}
									<span class="ml-1 text-xs font-normal opacity-70"
										>({OFFLINE_STATUS_LABEL})</span
									>
								{/if}
							</div>
							<div class="flex items-center gap-1.5">
								{#if isModerator || observerName === name}
									<button
										class="btn variant-filled px-2 py-0.5 text-xs"
										on:click={() => setObserver(observerName, false)}
									>
										{observerJoinActionLabel(info)}
									</button>
								{/if}
								{#if isModerator}
									<button
										class="btn variant-filled px-2 py-0.5 text-xs"
										on:click={() => kickPlayer(observerName)}
									>
										Kick
									</button>
								{/if}
							</div>
						</div>
					</div>
				{/each}
			</div>
			{#if stage !== 'Joining'}
				<div class="mt-3 rounded border border-white/20 px-2 py-2">
					<label class="flex items-start gap-3 text-sm">
						<input
							type="checkbox"
							class="mt-0.5 h-4 w-4 cursor-pointer accent-primary-500"
							checked={allowNewPlayersMidgame}
							on:change={updateAllowMidgameJoin}
						/>
						<span>Allow new players to join</span>
					</label>
				</div>
				<div class="mt-3 rounded border border-white/20 px-2 py-2">
					<p class="block text-sm font-semibold">Storyteller loss complement (C)</p>
					<p class="mt-1 text-xs opacity-75">
						Loss triggers when at least (guessers − C) are right or wrong.
					</p>
					<div class="mt-2 flex items-center gap-2">
						<input
							type="number"
							class="w-24 rounded border px-2 py-1 text-gray-700 shadow"
							min={storytellerLossComplementMin}
							max={storytellerLossComplementMax}
							step="1"
							value={storytellerLossComplement}
							on:change={updateStorytellerLossComplement}
							disabled={!isModerator}
						/>
						<span class="text-xs opacity-75"
							>Range: {storytellerLossComplementMin}–{storytellerLossComplementMax}</span
						>
					</div>
				</div>
				<div class="mt-3 rounded border border-white/20 px-2 py-2">
					<p class="block text-sm font-semibold">Votes per guesser</p>
					<p class="mt-1 text-xs opacity-75">
						How many vote tokens each guesser can cast in voting.
					</p>
					<div class="mt-2 flex items-center gap-2">
						<input
							type="number"
							class="w-24 rounded border px-2 py-1 text-gray-700 shadow"
							min={votesPerGuesserMin}
							max={votesPerGuesserMax}
							step="1"
							value={votesPerGuesser}
							on:change={updateVotesPerGuesser}
							disabled={!isModerator || !canChangePreVotingSettings}
						/>
						<span class="text-xs opacity-75"
							>Range: {votesPerGuesserMin}–{votesPerGuesserMax}</span
						>
					</div>
					{#if !canChangePreVotingSettings}
						<p class="mt-1 text-xs opacity-70">
							Can only be changed during storyteller choosing stage.
						</p>
					{/if}
				</div>
				<div class="mt-3 rounded border border-white/20 px-2 py-2">
					<p class="block text-sm font-semibold">Cards per hand</p>
					<div class="mt-2 flex items-center gap-2">
						<input
							type="number"
							class="w-24 rounded border px-2 py-1 text-gray-700 shadow"
							min={cardsPerHandMin}
							max={cardsPerHandMax}
							step="1"
							value={cardsPerHand}
							on:change={updateCardsPerHand}
							disabled={!isModerator || !canChangeCardsPerHand}
						/>
						<span class="text-xs opacity-75">Range: {cardsPerHandMin}–{cardsPerHandMax}</span>
					</div>
					{#if !canChangeCardsPerHand}
						<p class="mt-1 text-xs opacity-70">
							Can only be changed at round start (before clue).
						</p>
					{/if}
				</div>
				<div class="mt-3 rounded border border-white/20 px-2 py-2">
					<p class="block text-sm font-semibold">Nominations per guesser</p>
					<div class="mt-2 flex items-center gap-2">
						<input
							type="number"
							class="w-24 rounded border px-2 py-1 text-gray-700 shadow"
							min={nominationsPerGuesserMin}
							max={nominationsPerGuesserMax}
							step="1"
							value={nominationsPerGuesser}
							on:change={updateNominationsPerGuesser}
							disabled={!isModerator || !canChangePreVotingSettings}
						/>
						<span class="text-xs opacity-75"
							>Range: {nominationsPerGuesserMin}–{nominationsPerGuesserMax}</span
						>
					</div>
					{#if !canChangePreVotingSettings}
						<p class="mt-1 text-xs opacity-70">
							Can only be changed during storyteller choosing stage.
						</p>
					{/if}
				</div>
			{/if}
		</details>
	{/if}
</div>
