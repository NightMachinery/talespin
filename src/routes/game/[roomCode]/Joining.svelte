<script lang="ts">
	import { browser } from '$app/environment';
	import type GameServer from '$lib/gameServer';
	import {
		BUILTIN_STELLA_WORD_PACK_PRESETS,
		type StellaWordPackPreset
	} from '$lib/stellaWordPacks';
	import type { GameMode, PlayerInfo, WinCondition } from '$lib/types';
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
	export let cardsPerHand = 12;
	export let cardsPerHandMin = 1;
	export let cardsPerHandMax = 100;
	export let stellaBoardSize = 15;
	export let stellaBoardSizeMin = 1;
	export let stellaBoardSizeMax = 100;
	export let stellaSelectionMin = 1;
	export let stellaSelectionMax = 10;
	export let stellaSelectionCountMin = 1;
	export let stellaSelectionCountMax = 15;
	export let stellaWordPackSize = 0;

	const toastStore = getToastStore();
	const LOCAL_STELLA_PACKS_KEY = 'stella_word_pack_presets';
	const LOCAL_PRESET_KEY_PREFIX = 'local:';
	const BUILTIN_PRESET_KEY_PREFIX = 'builtin:';
	const WORD_PACK_APPLY_DEBOUNCE_MS = 350;
	$: playerEntries = Object.entries(players);
	$: moderatorSet = new Set(moderators);
	$: isCreator = creator !== '' && creator === name;
	$: isModerator = moderatorSet.has(name);
	$: canStart = roomStateLoaded && isModerator && playerEntries.length >= 3;
	$: canEditSettings = roomStateLoaded && isModerator && stage === 'Joining';
	$: showSettings = roomStateLoaded;
	$: activeWinMode = winCondition.mode;
	let localTargetPoints = 10;
	let localTargetCycles = 3;
	let localTargetRounds = 4;
	let localWordPackText = '';
	let newPresetName = '';
	let savedPresets: StellaWordPackPreset[] = [];
	let selectedPresetKey = '';
	let wordPackApplyTimeout: ReturnType<typeof setTimeout> | null = null;
	let availableWordPackPresets: StellaWordPackOption[] = [];
	let selectedWordPackPreset: StellaWordPackOption | null = null;

	type StellaWordPackOption = StellaWordPackPreset & {
		key: string;
		builtin: boolean;
	};

	$: availableWordPackPresets = [
		...BUILTIN_STELLA_WORD_PACK_PRESETS.map((preset) => ({
			...preset,
			key: `${BUILTIN_PRESET_KEY_PREFIX}${preset.name}`,
			builtin: true
		})),
		...savedPresets.map((preset) => ({
			...preset,
			key: `${LOCAL_PRESET_KEY_PREFIX}${preset.name}`,
			builtin: false
		}))
	].sort((a, b) => a.name.localeCompare(b.name));
	$: selectedWordPackPreset =
		availableWordPackPresets.find((preset) => preset.key === selectedPresetKey) ?? null;

	$: if (winCondition.mode === 'points') localTargetPoints = winCondition.target_points;
	$: if (winCondition.mode === 'cycles') localTargetCycles = winCondition.target_cycles;
	$: if (winCondition.mode === 'fixed_rounds') localTargetRounds = winCondition.target_rounds;

	function loadPresets() {
		if (!browser) return;
		try {
			const parsed = JSON.parse(window.localStorage.getItem(LOCAL_STELLA_PACKS_KEY) || '[]');
			savedPresets = Array.isArray(parsed)
				? parsed.filter(
						(entry): entry is StellaWordPackPreset =>
							typeof entry?.name === 'string' && typeof entry?.words === 'string'
					)
				: [];
		} catch {
			savedPresets = [];
		}
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

	function normalizeWordPackText(rawWords: string) {
		return rawWords
			.split(/\r?\n/u)
			.map((word) => word.trim())
			.filter((word) => word.length > 0)
			.join('\n');
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

	function getWordPackPresetByKey(key: string) {
		return availableWordPackPresets.find((preset) => preset.key === key) ?? null;
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
		const preset = getWordPackPresetByKey(nextKey);
		if (preset) {
			updateSelectedWordPack(preset.words);
		}
	}

	function deleteSelectedPreset() {
		if (!selectedWordPackPreset || selectedWordPackPreset.builtin) return;
		savedPresets = savedPresets.filter((preset) => preset.name !== selectedWordPackPreset.name);
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
				<div class="flex justify-center mt-4 lg:justify-start">
					<button class="btn variant-filled" disabled={!roomStateLoaded} on:click={copyInviteLink}
						>Copy Invite Link</button
					>
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
							<option value="dixit_plus">Dixit+</option>
							<option value="stella">Stella</option>
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
							<option value="cycles">Cycles</option>
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
							<p class="mt-1 text-xs opacity-70">Range: {cardsPerHandMin}–{cardsPerHandMax}</p>
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
							<div class="rounded border border-white/20 p-3 space-y-2">
								<div class="flex items-center justify-between gap-2">
									<h3 class="font-semibold">Word pack</h3>
									<span class="text-xs opacity-70">Current pack size: {stellaWordPackSize}</span>
								</div>
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
											<option value={preset.key}>
												{preset.name}{preset.builtin ? ' (built-in)' : ''}
											</option>
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
										disabled={!selectedWordPackPreset || selectedWordPackPreset.builtin}
										>Delete</button
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
