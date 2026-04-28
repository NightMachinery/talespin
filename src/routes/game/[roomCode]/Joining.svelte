<script lang="ts">
	import { browser } from '$app/environment';
	import MigrateDeviceButton from '$lib/MigrateDeviceButton.svelte';
	import {
		clueRatingEnabled,
		clueRatingMaxStars,
		clueRatingMaxStarsMax,
		clueRatingMaxStarsMin
	} from '$lib/clueRating';
	import { copyTextToClipboard } from '$lib/deviceMigration';
	import type GameServer from '$lib/gameServer';
	import {
		BUILTIN_STELLA_WORD_PACK_PRESETS,
		LOCAL_PRESET_KEY_PREFIX,
		LOCAL_STELLA_PACKS_KEY,
		buildStellaWordPackOptions,
		findStellaWordPackOptionByWords,
		formatStellaWordPackOptionLabel,
		normalizeWordPackText,
		parseSavedStellaWordPackPresets,
		type StellaWordPackOption,
		type StellaWordPackPreset
	} from '$lib/stellaWordPacks';
	import type {
		GameMode,
		PlayerInfo,
		StellaQueuedRevealMode,
		WinCondition
	} from '$lib/types';
	import { formatWinCondition } from '$lib/winCondition';
	import { Avatar, getToastStore } from '@skeletonlabs/skeleton';
	import { onDestroy } from 'svelte';

	export let players: { [key: string]: PlayerInfo } = {};
	export let roomCode = '';
	export let gameServer: GameServer;
	export let name = '';
	export let roomStateLoaded = false;
	export let creator = '';
	export let moderators: string[] = [];
	export let stage = 'Joining';
	export let gameMode: GameMode = 'dixit_plus';
	export let winCondition: WinCondition = { mode: 'cards_finish' };
	export let storytellerPoolEnabled = false;
	export let storytellerPoolActive = false;
	export let storytellerPoolPlayers: string[] = [];
	export let storytellerSuccessPoints = 3;
	export let storytellerSuccessPointsMin = 0;
	export let storytellerSuccessPointsMax = 10;
	export let cardsPerHand = 12;
	export let cardsPerHandMin = 1;
	export let cardsPerHandMax = 100;
	export let beautyEnabled = false;
	export let beautyVotesPerPlayer = 2;
	export let beautyVotesPerPlayerMin = 1;
	export let beautyVotesPerPlayerMax = 1;
	export let beautyAllowDuplicateVotes = false;
	export let beautySplitPointsOnTie = true;
	export let beautyPointsBonus = 2;
	export let beautyPointsBonusMin = 0;
	export let beautyPointsBonusMax = 10;
	export let beautyResultsDisplayMode: import('$lib/types').BeautyResultsDisplayMode = 'combined';
	export let showPreviousResultsDuringStorytellerChoosing = true;
	export let stellaBoardSize = 15;
	export let stellaBoardSizeMin = 1;
	export let stellaBoardSizeMax = 100;
	export let stellaSelectionMin = 1;
	export let stellaSelectionMax = 10;
	export let stellaSelectionCountMin = 1;
	export let stellaSelectionCountMax = 15;
	export let stellaWordPackWords: string[] = [];
	export let stellaQueueDuringAssociation = true;
	export let stellaQueuedRevealMode: StellaQueuedRevealMode = 'animated';
	export let stellaScoutTimerEnabled = true;
	export let stellaScoutTimerDurationS = 10;
	export let forceStellaScoutTimer = false;

	const toastStore = getToastStore();
	const WORD_PACK_APPLY_DEBOUNCE_MS = 350;
	$: playerEntries = Object.entries(players);
	$: storytellerPoolPlayerSet = new Set(storytellerPoolPlayers);
	$: moderatorSet = new Set(moderators);
	$: isCreator = creator !== '' && creator === name;
	$: isModerator = moderatorSet.has(name);
	$: canStart = roomStateLoaded && isModerator && playerEntries.length >= 3;
	$: canEditSettings = roomStateLoaded && isModerator && stage === 'Joining';
	$: showSettings = roomStateLoaded;
	$: activeWinMode = winCondition.mode;
	let localTargetPoints = 10;
	let localTargetCycles = 1;
	let localTargetRounds = 6;
	let localWordPackText = '';
	let newPresetName = '';
	let savedPresets: StellaWordPackPreset[] = [];
	let selectedPresetKey = '';
	let wordPackApplyTimeout: number | null = null;
	let availableWordPackPresets: StellaWordPackOption[] = [];
	let selectedWordPackPreset: StellaWordPackOption | null = null;
	let lastServerWordPackText = '';
	$: availableWordPackPresets = buildStellaWordPackOptions(savedPresets, serverWordPackText);
	$: selectedWordPackPreset =
		availableWordPackPresets.find((preset) => preset.key === selectedPresetKey) ?? null;

	$: if (winCondition.mode === 'points') localTargetPoints = winCondition.target_points;
	$: if (winCondition.mode === 'cycles') localTargetCycles = winCondition.target_cycles;
	$: if (winCondition.mode === 'fixed_rounds') localTargetRounds = winCondition.target_rounds;
	$: serverWordPackText = normalizeWordPackText(stellaWordPackWords.join('\n'));
	$: if (roomStateLoaded && serverWordPackText !== lastServerWordPackText) {
		const normalizedLocalWordPackText = normalizeWordPackText(localWordPackText);
		const canSyncLocalWordPack =
			normalizedLocalWordPackText === '' || normalizedLocalWordPackText === lastServerWordPackText;
		if (canSyncLocalWordPack) {
			localWordPackText = serverWordPackText;
		}
		lastServerWordPackText = serverWordPackText;
	}
	$: if (roomStateLoaded && normalizeWordPackText(localWordPackText) === serverWordPackText) {
		selectedPresetKey =
			findStellaWordPackOptionByWords(availableWordPackPresets, serverWordPackText)?.key ?? '';
	}

	function loadPresets() {
		if (!browser) return;
		savedPresets = parseSavedStellaWordPackPresets(
			window.localStorage.getItem(LOCAL_STELLA_PACKS_KEY)
		);
	}

	function savePresets() {
		if (!browser) return;
		window.localStorage.setItem(LOCAL_STELLA_PACKS_KEY, JSON.stringify(savedPresets));
	}

	if (browser) {
		loadPresets();
	}

	function getInitialsFromString(playerName: string) {
		return playerName
			.split(' ')
			.map((n) => n[0])
			.join('');
	}

	function getInviteLink() {
		if (!browser) return '';
		const inviteUrl = new URL('/', window.location.origin);
		inviteUrl.searchParams.set('room', roomCode);
		return inviteUrl.toString();
	}

	async function copyInviteLink() {
		const inviteLink = getInviteLink();
		if (!inviteLink) return;
		await copyTextToClipboard(inviteLink);
		toastStore.trigger({ message: '🔗 Invite link copied', autohide: true, timeout: 2000 });
	}

	function startGame() {
		gameServer.startGame();
	}

	function isPlayerModerator(playerName: string) {
		return moderatorSet.has(playerName);
	}

	function kickPlayer(playerName: string) {
		if (!isModerator || playerName === name) return;
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

	function updateGameMode(event: Event) {
		const input = event.currentTarget as HTMLSelectElement;
		if (!canEditSettings) return;
		gameServer.setGameMode(input.value as GameMode);
	}

	function updateWinMode(event: Event) {
		const input = event.currentTarget as HTMLSelectElement;
		if (!canEditSettings) return;
		const mode = input.value as WinCondition['mode'];
		if (mode === 'points') {
			gameServer.setWinCondition({ mode, target_points: localTargetPoints });
		} else if (mode === 'cycles') {
			gameServer.setWinCondition({ mode, target_cycles: localTargetCycles });
		} else if (mode === 'fixed_rounds') {
			gameServer.setWinCondition({ mode, target_rounds: localTargetRounds });
		} else {
			gameServer.setWinCondition({ mode });
		}
	}

	function updateTargetValue(mode: WinCondition['mode'], value: number) {
		if (!canEditSettings) return;
		const normalized = Math.max(1, Math.floor(value || 1));
		if (mode === 'points') {
			gameServer.setWinCondition({ mode, target_points: normalized });
		} else if (mode === 'cycles') {
			gameServer.setWinCondition({ mode, target_cycles: normalized });
		} else if (mode === 'fixed_rounds') {
			gameServer.setWinCondition({ mode, target_rounds: normalized });
		}
	}

	function updatePointsTarget(event: Event) {
		updateTargetValue('points', Number((event.currentTarget as HTMLInputElement).value));
	}

	function updateCyclesTarget(event: Event) {
		updateTargetValue('cycles', Number((event.currentTarget as HTMLInputElement).value));
	}

	function updateRoundsTarget(event: Event) {
		updateTargetValue('fixed_rounds', Number((event.currentTarget as HTMLInputElement).value));
	}

	function updateCardsPerHand(event: Event) {
		const input = event.currentTarget as HTMLInputElement;
		const value = Number(input.value);
		if (!canEditSettings || !Number.isInteger(value)) return;
		gameServer.setCardsPerHand(value);
	}

	function updateStorytellerPoolEnabled(event: Event) {
		const input = event.currentTarget as HTMLInputElement;
		if (!canEditSettings || gameMode !== 'dixit_plus') return;
		gameServer.setStorytellerPoolEnabled(input.checked);
	}

	function updateStorytellerPoolPlayer(playerName: string, enabled: boolean) {
		if (!canEditSettings || gameMode !== 'dixit_plus') return;
		const nextPlayers = enabled
			? [...storytellerPoolPlayers, playerName]
			: storytellerPoolPlayers.filter((entryName) => entryName !== playerName);
		gameServer.setStorytellerPoolPlayers(nextPlayers);
	}

	function updateStorytellerPoolPlayerFromEvent(playerName: string, event: Event) {
		updateStorytellerPoolPlayer(playerName, (event.currentTarget as HTMLInputElement).checked);
	}

	function updateStorytellerSuccessPoints(event: Event) {
		const input = event.currentTarget as HTMLInputElement;
		const value = Number(input.value);
		if (!canEditSettings || !Number.isInteger(value)) return;
		gameServer.setStorytellerSuccessPoints(value);
	}

	function updateBeautyEnabled(event: Event) {
		const input = event.currentTarget as HTMLInputElement;
		if (!canEditSettings) return;
		gameServer.setBeautyEnabled(input.checked);
	}

	function updateBeautyVotesPerPlayer(event: Event) {
		const input = event.currentTarget as HTMLInputElement;
		const value = Number(input.value);
		if (!canEditSettings || !Number.isInteger(value)) return;
		gameServer.setBeautyVotesPerPlayer(value);
	}

	function updateBeautyAllowDuplicateVotes(event: Event) {
		const input = event.currentTarget as HTMLInputElement;
		if (!canEditSettings) return;
		gameServer.setBeautyAllowDuplicateVotes(input.checked);
	}

	function updateBeautySplitPointsOnTie(event: Event) {
		const input = event.currentTarget as HTMLInputElement;
		if (!canEditSettings) return;
		gameServer.setBeautySplitPointsOnTie(input.checked);
	}

	function updateBeautyPointsBonus(event: Event) {
		const input = event.currentTarget as HTMLInputElement;
		const value = Number(input.value);
		if (!canEditSettings || !Number.isInteger(value)) return;
		gameServer.setBeautyPointsBonus(value);
	}

	function updateClueRatingEnabled(event: Event) {
		const input = event.currentTarget as HTMLInputElement;
		if (!canEditSettings) return;
		gameServer.setClueRatingEnabled(input.checked);
	}

	function updateClueRatingMaxStars(event: Event) {
		const input = event.currentTarget as HTMLInputElement;
		const value = Number(input.value);
		if (
			!canEditSettings ||
			!Number.isInteger(value) ||
			value < $clueRatingMaxStarsMin ||
			value > $clueRatingMaxStarsMax
		) {
			return;
		}
		gameServer.setClueRatingMaxStars(value);
	}

	function updateShowPreviousResultsDuringStorytellerChoosing(event: Event) {
		const input = event.currentTarget as HTMLInputElement;
		if (!canEditSettings) return;
		gameServer.setShowPreviousResultsDuringStorytellerChoosing(input.checked);
	}

	function updateStellaBoardSize(event: Event) {
		const input = event.currentTarget as HTMLInputElement;
		const value = Number(input.value);
		if (!canEditSettings || !Number.isInteger(value)) return;
		gameServer.setStellaBoardSize(value);
	}

	function updateStellaSelectionMin(event: Event) {
		const input = event.currentTarget as HTMLInputElement;
		const value = Number(input.value);
		if (!canEditSettings || !Number.isInteger(value)) return;
		gameServer.setStellaSelectionMin(value);
	}

	function updateStellaSelectionMax(event: Event) {
		const input = event.currentTarget as HTMLInputElement;
		const value = Number(input.value);
		if (!canEditSettings || !Number.isInteger(value)) return;
		gameServer.setStellaSelectionMax(value);
	}

	function updateStellaQueueDuringAssociation(event: Event) {
		const input = event.currentTarget as HTMLInputElement;
		if (!canEditSettings) return;
		gameServer.setStellaQueueDuringAssociation(input.checked);
	}

	function updateStellaQueuedRevealMode(event: Event) {
		const input = event.currentTarget as HTMLSelectElement;
		if (!canEditSettings) return;
		gameServer.setStellaQueuedRevealMode(input.value as StellaQueuedRevealMode);
	}

	function updateStellaScoutTimerEnabled(event: Event) {
		const input = event.currentTarget as HTMLInputElement;
		if (!canEditSettings) return;
		gameServer.setStellaScoutTimerEnabled(input.checked);
	}

	function updateStellaScoutTimerDuration(event: Event) {
		const input = event.currentTarget as HTMLInputElement;
		const value = Number(input.value);
		if (!canEditSettings || !Number.isInteger(value) || value < 0) return;
		gameServer.setStellaScoutTimerDuration(value);
	}

	function updateForceStellaScoutTimer(event: Event) {
		const input = event.currentTarget as HTMLInputElement;
		if (!canEditSettings) return;
		gameServer.setForceStellaScoutTimer(input.checked);
	}

	function queueWordPackApply(words = localWordPackText) {
		if (!browser || !canEditSettings || gameMode !== 'stella') return;
		const normalizedWords = normalizeWordPackText(words);
		if (!normalizedWords) return;
		if (wordPackApplyTimeout) {
			clearTimeout(wordPackApplyTimeout);
		}
		wordPackApplyTimeout = window.setTimeout(() => {
			gameServer.setStellaWordPack(normalizedWords);
			wordPackApplyTimeout = null;
		}, WORD_PACK_APPLY_DEBOUNCE_MS);
	}

	function updateSelectedWordPack(words: string) {
		localWordPackText = words;
		queueWordPackApply(words);
	}

	function saveCurrentPreset() {
		if (!browser) return;
		const name = newPresetName.trim();
		const words = normalizeWordPackText(localWordPackText);
		if (!name || !words) return;
		if (BUILTIN_STELLA_WORD_PACK_PRESETS.some((preset) => preset.name === name)) {
			toastStore.trigger({
				message: `Preset name "${name}" is reserved for a built-in pack.`,
				autohide: true,
				timeout: 2500
			});
			return;
		}
		savedPresets = [...savedPresets.filter((preset) => preset.name !== name), { name, words }].sort(
			(a, b) => a.name.localeCompare(b.name)
		);
		savePresets();
		selectedPresetKey = `${LOCAL_PRESET_KEY_PREFIX}${name}`;
		localWordPackText = words;
		newPresetName = '';
	}

	function handlePresetSelectionChange(event: Event) {
		const nextKey = (event.currentTarget as HTMLSelectElement).value;
		selectedPresetKey = nextKey;
		const preset = availableWordPackPresets.find((option) => option.key === nextKey) ?? null;
		if (preset) {
			updateSelectedWordPack(preset.words);
		}
	}

	function deleteSelectedPreset() {
		const preset = selectedWordPackPreset;
		if (!preset || preset.builtin) return;
		savedPresets = savedPresets.filter((savedPreset) => savedPreset.name !== preset.name);
		savePresets();
		selectedPresetKey = '';
	}

	async function importWordPack(event: Event) {
		const input = event.currentTarget as HTMLInputElement;
		const file = input.files?.[0];
		if (!file) return;
		updateSelectedWordPack(await file.text());
		selectedPresetKey = '';
		input.value = '';
	}

	function exportSelectedPreset() {
		if (!browser) return;
		if (!selectedWordPackPreset) return;
		const blob = new Blob([selectedWordPackPreset.words], { type: 'text/plain;charset=utf-8' });
		const url = URL.createObjectURL(blob);
		const link = document.createElement('a');
		link.href = url;
		link.download = `${selectedWordPackPreset.name}.txt`;
		link.click();
		URL.revokeObjectURL(url);
	}

	onDestroy(() => {
		if (wordPackApplyTimeout) {
			clearTimeout(wordPackApplyTimeout);
		}
	});
</script>

<div class="m-auto w-full max-w-6xl px-4">
	<div class="grid gap-6 lg:grid-cols-[minmax(0,1fr)_340px]">
		<div>
			<div class="max-w-md mx-auto lg:mx-0">
				<h1 class="text-3xl text-center lg:text-left">Hi {name}, let's play Talespin!</h1>
				<h2 class="text-xl text-center lg:text-left">
					You are in room <code class="code text-lg">{roomCode}</code>
				</h2>
				<p class="mt-1 text-center text-sm opacity-70 lg:text-left">
					Players in lobby: {playerEntries.length}
				</p>
				<div class="mt-4 flex flex-wrap justify-center gap-2 lg:justify-start">
					<button class="btn variant-filled" disabled={!roomStateLoaded} on:click={copyInviteLink}
						>Copy Invite Link</button
					>
					<MigrateDeviceButton />
				</div>
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
											on:click={() => setModerator(playerName, false)}>Demote</button
										>
									{:else if (isCreator || isModerator) && !isPlayerModerator(playerName)}
										<button
											class="btn variant-filled px-3 py-1 text-sm"
											on:click={() => setModerator(playerName, true)}>Make Mod</button
										>
									{/if}
								{/if}
								{#if isModerator && playerName !== name}
									<button
										class="btn variant-filled px-3 py-1 text-sm"
										on:click={() => kickPlayer(playerName)}>Kick</button
									>
								{/if}
							</div>
						</div>
					</div>
				{/each}
			</div>
		</div>

		<div class="space-y-4">
			{#if showSettings}
				<div class="card p-4 space-y-4">
					<h2 class="text-xl font-semibold">Room settings</h2>
					<div>
						<label class="text-sm font-semibold" for="gameMode">Game mode</label>
						<select
							id="gameMode"
							class="mt-1 w-full rounded border px-3 py-2 text-gray-700"
							value={gameMode}
							on:change={updateGameMode}
							disabled={!canEditSettings}
						>
							<option value="dixit_plus">Talespin</option>
							<option value="stella">Resonance</option>
						</select>
					</div>
					<div>
						<label class="text-sm font-semibold" for="winMode">Win condition</label>
						<select
							id="winMode"
							class="mt-1 w-full rounded border px-3 py-2 text-gray-700"
							value={activeWinMode}
							on:change={updateWinMode}
							disabled={!canEditSettings}
						>
							<option value="cards_finish">Cards finish</option>
							<option value="points">Points</option>
							{#if gameMode !== 'stella'}
								<option value="cycles">Cycles</option>
							{/if}
							<option value="fixed_rounds">Fixed rounds</option>
						</select>
						{#if activeWinMode === 'points'}
							<input
								class="mt-2 w-full rounded border px-3 py-2 text-gray-700"
								type="number"
								min="1"
								value={localTargetPoints}
								on:change={updatePointsTarget}
								disabled={!canEditSettings}
							/>
						{:else if activeWinMode === 'cycles'}
							<input
								class="mt-2 w-full rounded border px-3 py-2 text-gray-700"
								type="number"
								min="1"
								value={localTargetCycles}
								on:change={updateCyclesTarget}
								disabled={!canEditSettings}
							/>
						{:else if activeWinMode === 'fixed_rounds'}
							<input
								class="mt-2 w-full rounded border px-3 py-2 text-gray-700"
								type="number"
								min="1"
								value={localTargetRounds}
								on:change={updateRoundsTarget}
								disabled={!canEditSettings}
							/>
						{/if}
						<p class="mt-2 text-xs opacity-70">Current: {formatWinCondition(winCondition)}</p>
					</div>

					{#if gameMode === 'dixit_plus'}
						<div class="space-y-3">
							<div>
								<label class="text-sm font-semibold" for="cardsPerHand">Cards per hand</label>
								<input
									id="cardsPerHand"
									class="mt-1 w-full rounded border px-3 py-2 text-gray-700"
									type="number"
									min={cardsPerHandMin}
									max={cardsPerHandMax}
									value={cardsPerHand}
									on:change={updateCardsPerHand}
									disabled={!canEditSettings}
								/>
								<p class="mt-1 text-xs opacity-70">
									Range: {cardsPerHandMin}–{cardsPerHandMax}
								</p>
							</div>
							<div class="rounded border border-white/20 p-3 space-y-3">
								<label class="flex items-start gap-3 text-sm">
									<input
										type="checkbox"
										class="mt-0.5 h-4 w-4 cursor-pointer accent-primary-500"
										checked={storytellerPoolEnabled}
										on:change={updateStorytellerPoolEnabled}
										disabled={!canEditSettings}
									/>
									<div>
										<span class="block font-semibold">Restrict storyteller pool</span>
										<p class="text-xs opacity-70">
											Only selected lobby players can be chosen as storyteller while at least one of
											them is active.
										</p>
									</div>
								</label>
								<div>
									<p class="text-sm font-semibold">Storyteller players</p>
									<div class="mt-2 space-y-2">
										{#each playerEntries as [playerName]}
											<label class="flex items-center gap-3 text-sm">
												<input
													type="checkbox"
													class="h-4 w-4 cursor-pointer accent-primary-500"
													checked={storytellerPoolPlayerSet.has(playerName)}
													on:change={(event) =>
														updateStorytellerPoolPlayerFromEvent(playerName, event)}
													disabled={!canEditSettings}
												/>
												<span>{playerName}</span>
											</label>
										{/each}
									</div>
									{#if storytellerPoolEnabled && !storytellerPoolActive}
										<p class="mt-2 text-xs opacity-70">
											Currently inactive. If all selected players become observers or are removed,
											the restriction pauses until one of them is active again.
										</p>
									{/if}
								</div>
							</div>
							<div>
								<label class="text-sm font-semibold" for="storytellerSuccessPoints">
									Storyteller success score
								</label>
								<input
									id="storytellerSuccessPoints"
									class="mt-1 w-full rounded border px-3 py-2 text-gray-700"
									type="number"
									min={storytellerSuccessPointsMin}
									max={storytellerSuccessPointsMax}
									value={storytellerSuccessPoints}
									on:change={updateStorytellerSuccessPoints}
									disabled={!canEditSettings}
								/>
								<p class="mt-1 text-xs opacity-70">
									Range: {storytellerSuccessPointsMin}–{storytellerSuccessPointsMax}
								</p>
							</div>
							<div class="rounded border border-white/20 p-3 space-y-3">
								<label class="flex items-start gap-3 text-sm">
									<input
										type="checkbox"
										class="mt-0.5 h-4 w-4 cursor-pointer accent-primary-500"
										checked={beautyEnabled}
										on:change={updateBeautyEnabled}
										disabled={!canEditSettings}
									/>
									<div>
										<span class="block font-semibold">Enable Most Beautiful</span>
										<p class="text-xs opacity-70">
											Add a second voting step after storyteller voting.
										</p>
									</div>
								</label>
								<div class="grid grid-cols-2 gap-3">
									<div>
										<label class="text-sm font-semibold" for="beautyVotesPerPlayer">
											Beauty votes
										</label>
										<input
											id="beautyVotesPerPlayer"
											class="mt-1 w-full rounded border px-3 py-2 text-gray-700"
											type="number"
											min={beautyVotesPerPlayerMin}
											max={beautyVotesPerPlayerMax}
											value={beautyVotesPerPlayer}
											on:change={updateBeautyVotesPerPlayer}
											disabled={!canEditSettings}
										/>
										<p class="mt-1 text-xs opacity-70">
											Range: {beautyVotesPerPlayerMin}–{beautyVotesPerPlayerMax}
										</p>
									</div>
									<div>
										<label class="text-sm font-semibold" for="beautyPointsBonus">
											Beauty bonus
										</label>
										<input
											id="beautyPointsBonus"
											class="mt-1 w-full rounded border px-3 py-2 text-gray-700"
											type="number"
											min={beautyPointsBonusMin}
											max={beautyPointsBonusMax}
											value={beautyPointsBonus}
											on:change={updateBeautyPointsBonus}
											disabled={!canEditSettings}
										/>
										<p class="mt-1 text-xs opacity-70">
											Range: {beautyPointsBonusMin}–{beautyPointsBonusMax}
										</p>
									</div>
								</div>
								<label class="flex items-start gap-3 text-sm">
									<input
										type="checkbox"
										class="mt-0.5 h-4 w-4 cursor-pointer accent-primary-500"
										checked={beautyAllowDuplicateVotes}
										on:change={updateBeautyAllowDuplicateVotes}
										disabled={!canEditSettings}
									/>
									<span>Allow duplicate beauty votes on the same card</span>
								</label>
								<label class="flex items-start gap-3 text-sm">
									<input
										type="checkbox"
										class="mt-0.5 h-4 w-4 cursor-pointer accent-primary-500"
										checked={beautySplitPointsOnTie}
										on:change={updateBeautySplitPointsOnTie}
										disabled={!canEditSettings}
									/>
									<span>Split beauty bonus among tied owners, rounding up</span>
								</label>
								<label class="flex items-start gap-3 text-sm">
									<input
										type="checkbox"
										class="mt-0.5 h-4 w-4 cursor-pointer accent-primary-500"
										checked={showPreviousResultsDuringStorytellerChoosing}
										on:change={updateShowPreviousResultsDuringStorytellerChoosing}
										disabled={!canEditSettings}
									/>
									<div>
										<span class="block font-semibold">
											Show previous results while storyteller is choosing and during nominations
										</span>
										<p class="text-xs opacity-70">
											Active players can locally switch between Previous Results and My Cards;
											observers see the preview-only version.
										</p>
									</div>
								</label>
								<div class="rounded border border-white/15 px-3 py-3">
									<label class="flex items-start gap-3 text-sm">
										<input
											type="checkbox"
											class="mt-0.5 h-4 w-4 cursor-pointer accent-primary-500"
											checked={$clueRatingEnabled}
											on:change={updateClueRatingEnabled}
											disabled={!canEditSettings}
										/>
										<div>
											<span class="block font-semibold">Enable clue rating</span>
											<p class="text-xs opacity-70">Add a star-vote stage before results.</p>
										</div>
									</label>
									<div class="mt-3">
										<label class="text-sm font-semibold" for="clueRatingMaxStars">
											Max stars
										</label>
										<input
											id="clueRatingMaxStars"
											class="mt-1 w-full rounded border px-3 py-2 text-gray-700"
											type="number"
											min={$clueRatingMaxStarsMin}
											max={$clueRatingMaxStarsMax}
											value={$clueRatingMaxStars}
											on:change={updateClueRatingMaxStars}
											disabled={!canEditSettings}
										/>
										<p class="mt-1 text-xs opacity-70">
											Range: {$clueRatingMaxStarsMin}–{$clueRatingMaxStarsMax}. Bonus =
											max(round(avg stars) - 1, 0).
										</p>
									</div>
								</div>
							</div>
						</div>
					{:else}
						<div class="space-y-3">
							<div>
								<label class="text-sm font-semibold" for="stellaBoardSize">Board size</label>
								<input
									id="stellaBoardSize"
									class="mt-1 w-full rounded border px-3 py-2 text-gray-700"
									type="number"
									min={stellaBoardSizeMin}
									max={stellaBoardSizeMax}
									value={stellaBoardSize}
									on:change={updateStellaBoardSize}
									disabled={!canEditSettings}
								/>
								<p class="mt-1 text-xs opacity-70">
									Range: {stellaBoardSizeMin}–{stellaBoardSizeMax}
								</p>
							</div>
							<div class="grid grid-cols-2 gap-3">
								<div>
									<label class="text-sm font-semibold" for="stellaSelectionMin">Selection min</label
									>
									<input
										id="stellaSelectionMin"
										class="mt-1 w-full rounded border px-3 py-2 text-gray-700"
										type="number"
										min={stellaSelectionCountMin}
										max={stellaSelectionCountMax}
										value={stellaSelectionMin}
										on:change={updateStellaSelectionMin}
										disabled={!canEditSettings}
									/>
								</div>
								<div>
									<label class="text-sm font-semibold" for="stellaSelectionMax">Selection max</label
									>
									<input
										id="stellaSelectionMax"
										class="mt-1 w-full rounded border px-3 py-2 text-gray-700"
										type="number"
										min={stellaSelectionCountMin}
										max={stellaSelectionCountMax}
										value={stellaSelectionMax}
										on:change={updateStellaSelectionMax}
										disabled={!canEditSettings}
									/>
								</div>
							</div>
							<div class="rounded border border-white/20 p-3 space-y-3">
								<label class="flex items-start gap-3 text-sm">
									<input
										type="checkbox"
										class="mt-0.5 h-4 w-4 cursor-pointer accent-primary-500"
										checked={stellaQueueDuringAssociation}
										on:change={updateStellaQueueDuringAssociation}
										disabled={!canEditSettings}
									/>
									<div>
										<span class="block font-semibold">Queue during Association</span>
										<p class="text-xs opacity-70">
											Players order reveals while selecting cards. This is the default.
										</p>
									</div>
								</label>
								<div>
									<label class="text-sm font-semibold" for="stellaQueuedRevealMode">
										Queued reveal playback
									</label>
									<select
										id="stellaQueuedRevealMode"
										class="mt-1 w-full rounded border px-3 py-2 text-gray-700"
										value={stellaQueuedRevealMode}
										on:change={updateStellaQueuedRevealMode}
										disabled={!canEditSettings || !stellaQueueDuringAssociation}
									>
										<option value="animated">Animated reveal</option>
										<option value="fast">Fast reveal</option>
									</select>
								</div>
								<div class="rounded border border-white/15 p-3 space-y-3">
									<p class="font-semibold">Manual reveal fallback</p>
									<p class="text-xs opacity-70">
										These only apply when Queue during Association is turned off.
									</p>
									<label class="flex items-center gap-3 text-sm">
										<input
											type="checkbox"
											class="h-4 w-4 cursor-pointer accent-primary-500"
											checked={stellaScoutTimerEnabled}
											on:change={updateStellaScoutTimerEnabled}
											disabled={!canEditSettings}
										/>
										<span>Scout timer enabled</span>
									</label>
									<div>
										<label class="text-sm font-semibold" for="stellaScoutTimerDuration">
											Scout timer duration
										</label>
										<input
											id="stellaScoutTimerDuration"
											class="mt-1 w-full rounded border px-3 py-2 text-gray-700"
											type="number"
											min="0"
											max="3600"
											value={stellaScoutTimerDurationS}
											on:change={updateStellaScoutTimerDuration}
											disabled={!canEditSettings}
										/>
										<p class="mt-1 text-xs opacity-70">0 = auto-resolve manual reveal mode.</p>
									</div>
									<label class="flex items-start gap-3 text-sm">
										<input
											type="checkbox"
											class="mt-0.5 h-4 w-4 cursor-pointer accent-primary-500"
											checked={forceStellaScoutTimer}
											on:change={updateForceStellaScoutTimer}
											disabled={!canEditSettings}
										/>
										<span>Force timeout by auto-revealing a random queued card</span>
									</label>
								</div>
							</div>
							<div class="rounded border border-white/20 p-3 space-y-2">
								<h3 class="font-semibold">Word pack</h3>
								<textarea
									class="w-full rounded border px-3 py-2 text-gray-700 min-h-[160px]"
									bind:value={localWordPackText}
									placeholder="One clue word per line"
									on:input={() => queueWordPackApply()}
									disabled={!canEditSettings}
								></textarea>
								<p class="text-xs opacity-70">Changes apply automatically after you stop typing.</p>
								<div class="flex flex-wrap gap-2">
									<input
										type="file"
										accept=".txt,text/plain"
										on:change={importWordPack}
										disabled={!canEditSettings}
									/>
								</div>
								<div class="grid grid-cols-[1fr_auto] gap-2">
									<input
										class="rounded border px-3 py-2 text-gray-700"
										placeholder="Preset name"
										bind:value={newPresetName}
									/>
									<button class="btn variant-filled" on:click={saveCurrentPreset}
										>Save preset</button
									>
								</div>
								<div class="flex flex-wrap gap-2 items-center">
									<select
										class="rounded border px-3 py-2 text-gray-700 min-w-[180px]"
										value={selectedPresetKey}
										on:change={handlePresetSelectionChange}
									>
										<option value="">Choose a word pack preset</option>
										{#each availableWordPackPresets as preset}
											<option value={preset.key}>{formatStellaWordPackOptionLabel(preset)}</option>
										{/each}
									</select>
									<button
										class="btn variant-filled"
										on:click={exportSelectedPreset}
										disabled={!selectedWordPackPreset}>Export</button
									>
									<button
										class="btn variant-filled"
										on:click={deleteSelectedPreset}
										disabled={!selectedWordPackPreset ||
											selectedWordPackPreset.builtin ||
											selectedWordPackPreset.unsaved}>Delete</button
									>
								</div>
							</div>
						</div>
					{/if}

					{#if !canEditSettings}
						<p class="text-xs opacity-70">
							Only moderators can edit room settings. Everyone else sees the live room config here.
						</p>
					{/if}
				</div>
			{/if}

			<div class="card p-4 space-y-2">
				{#if isModerator}
					<button class="btn variant-filled w-full" disabled={!canStart} on:click={startGame}
						>Start Game</button
					>
					{#if playerEntries.length < 3}
						<p class="text-center text-sm opacity-70">Need at least 3 players to start.</p>
					{/if}
				{:else}
					<p class="text-center text-sm opacity-70">Waiting for a moderator to start the game.</p>
				{/if}
			</div>
		</div>
	</div>
</div>
