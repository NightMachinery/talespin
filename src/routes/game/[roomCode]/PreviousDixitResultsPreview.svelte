<script lang="ts">
	import { buildBeautyBadgeMetadata } from '$lib/beautyResults';
	import { getDesktopFitRowCount } from '$lib/cardGrid';
	import { CARD_IMAGE_ALT_TEXT } from '$lib/cardImageText';
	import CardImage from '$lib/CardImage.svelte';
	import { http_host } from '$lib/gameServer';
	import type { PreviousDixitResultsView } from '$lib/types';
	import { cardsFitToHeight } from '$lib/viewOptions';
	import ChooserNameOverlay from './ChooserNameOverlay.svelte';

	export let snapshot: PreviousDixitResultsView;
	export let showVotingCardNumbers = true;

	type OverlayEntry = { name: string; count?: number };

	let cardToPlayer: Record<string, string> = {};
	let storyCardToChooserEntries: Record<string, OverlayEntry[]> = {};
	let beautyCardToChooserEntries: Record<string, OverlayEntry[]> = {};
	let highlightedCardSet = new Set<string>();
	let activeCard = '';
	let beautyBadges: ReturnType<typeof buildBeautyBadgeMetadata> = {};

	$: resultsDesktopFitEnabled = $cardsFitToHeight;
	$: resultsDesktopFitClass = resultsDesktopFitEnabled ? 'lg:h-full' : '';
	$: resultsDesktopRowCount = getDesktopFitRowCount(snapshot.center_cards?.length ?? 0);
	$: resultsSectionClass = resultsDesktopFitEnabled
		? 'results-fit-grid grid w-full grid-cols-2 gap-3 overflow-y-auto lg:min-h-0 lg:flex-1 lg:grid-cols-3 lg:content-stretch'
		: 'grid w-full grid-cols-2 gap-3 overflow-y-auto lg:grid-cols-3 lg:auto-rows-max lg:content-start';
	$: resultsDesktopFitStyle = resultsDesktopFitEnabled
		? `--results-desktop-rows: ${resultsDesktopRowCount};`
		: '';
	$: resultsCardClass = (isHighlighted: boolean) =>
		`${isHighlighted ? 'boujee-border' : ''} relative overflow-hidden rounded-lg bg-slate-900/35 ${resultsDesktopFitClass}`;
	$: resultsImageClass = `relative w-full object-cover object-center aspect-[2/3] ${resultsDesktopFitClass}`;

	function buildChooserEntries(voteMap: Record<string, string[]>) {
		const cardToVoterCounts: Record<string, Record<string, number>> = {};

		for (const [voter, votes] of Object.entries(voteMap)) {
			for (const votedCard of votes || []) {
				cardToVoterCounts[votedCard] ??= {};
				cardToVoterCounts[votedCard][voter] = (cardToVoterCounts[votedCard][voter] ?? 0) + 1;
			}
		}

		return Object.fromEntries(
			Object.entries(cardToVoterCounts).map(([card, voterCounts]) => [
				card,
				Object.entries(voterCounts)
					.sort(([a], [b]) => a.localeCompare(b))
					.map(([voter, count]) => ({
						name: voter,
						...(count > 1 ? { count } : {})
					}))
			])
		) as Record<string, OverlayEntry[]>;
	}

	$: {
		cardToPlayer = {};
		for (const [playerName, cards] of Object.entries(snapshot.player_to_current_cards ?? {})) {
			for (const card of cards || []) {
				cardToPlayer[card] = playerName;
			}
		}

		storyCardToChooserEntries =
			snapshot.kind === 'results' ? buildChooserEntries(snapshot.player_to_votes) : {};
		beautyCardToChooserEntries = buildChooserEntries(snapshot.player_to_beauty_votes ?? {});
		beautyBadges = buildBeautyBadgeMetadata(snapshot.beauty_vote_totals ?? {});
		activeCard = snapshot.kind === 'results' ? snapshot.active_card : '';
		highlightedCardSet = new Set(
			snapshot.kind === 'results'
				? [snapshot.active_card, ...(snapshot.beauty_winning_cards ?? [])]
				: [...(snapshot.beauty_winning_cards ?? [])]
		);
	}
</script>

<div class="flex h-full min-h-0 flex-col">
	<h2 class="mb-2 hidden text-lg font-semibold lg:block">Previous round cards</h2>
	<section class={resultsSectionClass} style={resultsDesktopFitStyle}>
		{#each snapshot.center_cards as image, cardIndex}
			<div class={resultsCardClass(highlightedCardSet.has(image))}>
				<CardImage
					src={`${http_host}/cards/${image}`}
					alt={CARD_IMAGE_ALT_TEXT}
					className={resultsImageClass}
				/>
				{#if showVotingCardNumbers}
					<div
						class="absolute left-2 top-2 z-20 rounded bg-black/70 px-2 py-0.5 text-xs font-bold text-white shadow"
					>
						#{cardIndex + 1}
					</div>
				{/if}
				{#if snapshot.kind === 'results' && storyCardToChooserEntries[image]}
					<ChooserNameOverlay
						entries={storyCardToChooserEntries[image]}
						label={beautyBadges[image] && snapshot.beauty_results_display_mode === 'combined'
							? 'Guess'
							: ''}
						avoidTopLeftBadge={showVotingCardNumbers}
						tone="story"
					/>
				{/if}
				{#if snapshot.kind === 'results' && snapshot.beauty_results_display_mode === 'summary' && beautyBadges[image]}
					<ChooserNameOverlay
						label={beautyBadges[image].label}
						labelTier={beautyBadges[image].tier}
						position="bottom-left"
						tone="beauty"
					/>
				{:else if beautyBadges[image]}
					<ChooserNameOverlay
						entries={beautyCardToChooserEntries[image] ?? []}
						label={beautyBadges[image].label}
						labelTier={beautyBadges[image].tier}
						position={snapshot.kind === 'results' ? 'bottom-right' : 'top-right'}
						avoidTopLeftBadge={snapshot.kind !== 'results' && showVotingCardNumbers}
						tone="beauty"
					/>
				{/if}
				<div
					class={`absolute bottom-0 w-full rounded-tr bg-primary-200 p-0.5 px-2 font-bold text-black ${
						image === activeCard ? 'boujee-text' : ''
					}`}
				>
					{cardToPlayer[image]}'s card
				</div>
			</div>
		{/each}
	</section>
</div>

<style>
	@property --bg-angle {
		inherits: false;
		initial-value: 0deg;
		syntax: '<angle>';
	}

	.boujee-border {
		animation: spin 2.5s infinite linear;
		background:
			linear-gradient(to bottom, rgb(var(--color-primary-500)), rgb(var(--color-primary-500)))
				padding-box,
			conic-gradient(from var(--bg-angle) in oklch longer hue, rgb(var(--color-success-500)) 0 0)
				border-box;
		border: 5px solid transparent;
		box-shadow: 0.125rem 0.25rem 0.25rem 0.5rem oklch(0.1 0.37 315 / 0.25);
	}

	@keyframes spin {
		to {
			--bg-angle: 360deg;
		}
	}

	@media (min-width: 1024px) {
		.results-fit-grid {
			grid-template-rows: repeat(var(--results-desktop-rows, 2), minmax(0, 1fr));
		}
	}
</style>
