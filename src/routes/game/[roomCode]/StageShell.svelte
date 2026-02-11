<script lang="ts">
	import { cardsFitToHeight } from '$lib/viewOptions';
	import type GameServer from '$lib/gameServer';
	import Leaderboard from './Leaderboard.svelte';
	import SidebarOptions from './SidebarOptions.svelte';
	import type { ObserverInfo, PlayerInfo, WinCondition } from '$lib/types';

	export let players: { [key: string]: PlayerInfo } = {};
	export let observers: { [key: string]: ObserverInfo } = {};
	export let name = '';
	export let creator = '';
	export let moderators: string[] = [];
	export let gameServer: GameServer;
	export let stage = '';
	export let allowNewPlayersMidgame = true;
	export let storytellerLossThreshold = 1;
	export let storytellerLossThresholdMin = 1;
	export let storytellerLossThresholdMax = 1;
	export let votesPerGuesser = 1;
	export let votesPerGuesserMin = 1;
	export let votesPerGuesserMax = 1;
	export let activePlayer = '';
	export let pointChange: { [key: string]: number } = {};
	export let roundNum = 0;
	export let cardsRemaining = 0;
	export let deckRefillFlashToken = 0;
	export let winCondition: WinCondition = {
		mode: 'points',
		target_points: 10
	};
	export let showMobileActions = true;

	const hasMobileTop = !!$$slots.mobileTop;
	const hasMobileActions = !!$$slots.mobileActions;
	const hasMobileBottom = !!$$slots.mobileBottom;
	$: showMobileOptions = true;
	$: mainContentClass = `rounded-lg bg-black/10 p-2 sm:p-3 lg:p-4 ${
		$cardsFitToHeight ? 'lg:h-full' : ''
	}`;
</script>

<div class="w-full px-3 pt-3 lg:px-6 lg:pt-4">
	<div class="mx-auto max-w-[1400px]">
		{#if hasMobileTop}
			<div class="mb-3 lg:hidden">
				<slot name="mobileTop" />
			</div>
		{/if}

		<div class="flex flex-col gap-4 lg:h-[calc(100vh-2rem)] lg:flex-row">
			<main class="order-1 min-h-[58vh] flex-1 lg:order-2 lg:min-h-0">
				<div class={mainContentClass}>
					<slot />
				</div>
			</main>

			{#if hasMobileActions && showMobileActions}
				<div class="order-2 sticky bottom-2 z-20 lg:hidden">
					<div class="card light p-3">
						<slot name="mobileActions" />
					</div>
				</div>
			{/if}

			<aside class="order-3 w-full lg:order-1 lg:w-[340px] lg:shrink-0">
				<div
					class="flex flex-col gap-4 lg:sticky lg:top-4 lg:max-h-[calc(100vh-2rem)] lg:overflow-y-auto lg:pr-1"
				>
					<Leaderboard
						{players}
						{observers}
						{stage}
						{pointChange}
						{activePlayer}
						{roundNum}
						{cardsRemaining}
						{deckRefillFlashToken}
						{winCondition}
					/>

					<div class="hidden lg:block">
						<slot name="leftRail" />
					</div>

					{#if hasMobileBottom}
						<div class="lg:hidden">
							<slot name="mobileBottom" />
						</div>
					{/if}

					{#if showMobileOptions}
						<div class="lg:hidden">
							<SidebarOptions
								{players}
								{observers}
								{name}
								{creator}
								{moderators}
								{gameServer}
								{stage}
								{allowNewPlayersMidgame}
								{storytellerLossThreshold}
								{storytellerLossThresholdMin}
								{storytellerLossThresholdMax}
								{votesPerGuesser}
								{votesPerGuesserMin}
								{votesPerGuesserMax}
								{activePlayer}
							/>
						</div>
					{/if}

					<div class="hidden lg:block">
						<SidebarOptions
							{players}
							{observers}
							{name}
							{creator}
							{moderators}
							{gameServer}
							{stage}
							{allowNewPlayersMidgame}
							{storytellerLossThreshold}
							{storytellerLossThresholdMin}
							{storytellerLossThresholdMax}
							{votesPerGuesser}
							{votesPerGuesserMin}
							{votesPerGuesserMax}
							{activePlayer}
						/>
					</div>
				</div>
			</aside>
		</div>
	</div>
</div>
