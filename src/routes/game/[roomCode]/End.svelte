<script lang="ts">
	import {
		leaderboardExcludeBeauty,
		memberBeautyPoints,
		setLeaderboardExcludeBeautyPreference
	} from '$lib/mostBeautiful';
	import type { GameMode, ObserverInfo, PlayerInfo } from '$lib/types';
	import { rankEntriesByPoints, type RankedPlayerEntry } from '$lib/ranking';
	import MostBeautifulStatsPanel from './MostBeautifulStatsPanel.svelte';

	export let players: { [key: string]: PlayerInfo } = {};
	export let observers: { [key: string]: ObserverInfo } = {};
	export let gameMode: GameMode = 'dixit_plus';
	let rankedPlayers: RankedPlayerEntry[] = [];
	let sortedObserverEntries: Array<{
		name: string;
		displayPoints: number | null;
	}> = [];

	$: {
		rankedPlayers = rankEntriesByPoints(
			Object.entries(players).map(([name, info]) => ({
				name,
				points:
					info.points - ($leaderboardExcludeBeauty ? ($memberBeautyPoints[name] ?? 0) : 0)
			}))
		);
	}
	$: sortedObserverEntries = Object.entries(observers)
		.sort(([a], [b]) => a.localeCompare(b))
		.map(([name, info]) => ({
			name,
			displayPoints:
				info.points === null
					? null
					: info.points - ($leaderboardExcludeBeauty ? ($memberBeautyPoints[name] ?? 0) : 0)
		}));

	function handleExcludeBeautyToggle(event: Event) {
		const input = event.currentTarget as HTMLInputElement;
		setLeaderboardExcludeBeautyPreference(input.checked);
	}
</script>

<div class="flex w-80/10 justify-center">
	<div>
		<h1 class="text-4xl mb-10 text-center">Game Over!</h1>

		<div class="light p-4">
			<label class="mb-4 flex items-start gap-3 text-sm">
				<input
					type="checkbox"
					class="mt-0.5 h-4 w-4 cursor-pointer accent-primary-500"
					checked={$leaderboardExcludeBeauty}
					on:change={handleExcludeBeautyToggle}
				/>
				<span>Exclude beauty votes from leaderboard</span>
			</label>
			<div>
				{#each rankedPlayers as entry}
					<div class="flex space-between w-44 text-xl">
						<div class="flex-auto">
							{entry.rank}.
							<span class={`${entry.isTopScore ? 'boujee-text' : ''} `}>{entry.name}</span>
						</div>
						<div class="font-right">
							{entry.points}
						</div>
					</div>
				{/each}
			</div>
			{#if sortedObserverEntries.length > 0}
				<div class="mt-4 border-t border-white/15 pt-3">
					<h2 class="mb-2 text-lg font-semibold">Observers</h2>
					{#each sortedObserverEntries as observerEntry}
						<div class="flex space-between w-44 text-xl">
							<div class="flex-auto">{observerEntry.name}</div>
							<div class="font-right">
								{observerEntry.displayPoints === null ? 'NA' : observerEntry.displayPoints}
							</div>
						</div>
					{/each}
				</div>
			{/if}
		</div>
		{#if gameMode === 'dixit_plus'}
			<div class="mt-4">
				<MostBeautifulStatsPanel title="Most Beautiful ranking" />
			</div>
		{/if}
	</div>
</div>
