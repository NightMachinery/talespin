<script lang="ts">
	import { browser } from '$app/environment';
	import { onDestroy } from 'svelte';

	import { buildCardNumberNavigatorTargetId } from '$lib/cardNumberNavigator';
	import {
		stickyVotingCardNavigatorCollapsed,
		stickyVotingCardNavigatorEnabled
	} from '$lib/viewOptions';

	export let cardNumberLabels: number[] = [];
	export let targetIdScope = '';
	export let selectedLabels: number[] = [];
	export let mutedLabels: number[] = [];
	export let collapsedLabel = 'Card navigator';

	const TARGET_HIGHLIGHT_CLASS = 'card-number-navigator-target-highlight';
	let highlightTimeout: number | undefined;

	$: uniqueOrderedLabels = [...new Set(cardNumberLabels)]
		.filter((label): label is number => Number.isFinite(label) && label > 0)
		.sort((a, b) => a - b);
	$: selectedLabelSet = new Set(selectedLabels);
	$: mutedLabelSet = new Set(mutedLabels);
	$: shouldShowNavigator = $stickyVotingCardNavigatorEnabled && uniqueOrderedLabels.length > 1;
	$: toggleNavigatorLabel = `${
		$stickyVotingCardNavigatorCollapsed ? 'Show' : 'Collapse'
	} ${collapsedLabel}`;

	function prefersReducedMotion() {
		return browser && window.matchMedia('(prefers-reduced-motion: reduce)').matches;
	}

	function jumpToCard(label: number) {
		if (!browser) return;

		const targetId = buildCardNumberNavigatorTargetId(targetIdScope, label);
		const target = document.getElementById(targetId);
		if (!target) return;

		target.classList.remove(TARGET_HIGHLIGHT_CLASS);
		void target.getBoundingClientRect();
		target.classList.add(TARGET_HIGHLIGHT_CLASS);
		target.scrollIntoView({
			behavior: prefersReducedMotion() ? 'auto' : 'smooth',
			block: 'center',
			inline: 'nearest'
		});

		if (highlightTimeout) {
			window.clearTimeout(highlightTimeout);
		}
		highlightTimeout = window.setTimeout(() => {
			target.classList.remove(TARGET_HIGHLIGHT_CLASS);
		}, 1600);
	}

	onDestroy(() => {
		if (highlightTimeout) {
			window.clearTimeout(highlightTimeout);
		}
	});
</script>

{#if shouldShowNavigator}
	<div class="sticky top-2 z-50 mb-3">
		{#if $stickyVotingCardNavigatorCollapsed}
			<div class="flex justify-start">
				<button
					type="button"
					class="inline-flex h-9 w-9 items-center justify-center rounded-full border border-white/15 bg-slate-950/90 text-white shadow-lg backdrop-blur transition hover:bg-slate-900/95 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-white/80"
					aria-label={toggleNavigatorLabel}
					aria-expanded={!$stickyVotingCardNavigatorCollapsed}
					title={toggleNavigatorLabel}
					on:click={() => stickyVotingCardNavigatorCollapsed.set(false)}
				>
					<svg
						viewBox="0 0 24 24"
						class="h-4 w-4"
						fill="none"
						stroke="currentColor"
						stroke-linecap="round"
						stroke-linejoin="round"
						stroke-width="2"
						aria-hidden="true"
					>
						<path d="M6 9l6 6 6-6" />
					</svg>
					<span class="sr-only">{toggleNavigatorLabel}</span>
				</button>
			</div>
		{:else}
			<div
				class="relative rounded-2xl border border-white/10 bg-slate-950/90 px-2.5 py-2 shadow-lg backdrop-blur"
			>
				<button
					type="button"
					class="absolute right-2 top-2 inline-flex h-8 w-8 items-center justify-center rounded-full border border-white/15 bg-slate-950/70 text-white/80 transition hover:bg-white/10 hover:text-white focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-white/80"
					aria-label={toggleNavigatorLabel}
					aria-expanded={!$stickyVotingCardNavigatorCollapsed}
					title={toggleNavigatorLabel}
					on:click={() => stickyVotingCardNavigatorCollapsed.set(true)}
				>
					<svg
						viewBox="0 0 24 24"
						class="h-4 w-4"
						fill="none"
						stroke="currentColor"
						stroke-linecap="round"
						stroke-linejoin="round"
						stroke-width="2"
						aria-hidden="true"
					>
						<path d="M6 15l6-6 6 6" />
					</svg>
					<span class="sr-only">{toggleNavigatorLabel}</span>
				</button>
				<div
					class="card-number-navigator-grid -mx-0.5 flex flex-wrap gap-1.5 overflow-y-auto px-0.5 pr-10"
				>
					{#each uniqueOrderedLabels as label}
						{@const isSelected = selectedLabelSet.has(label)}
						{@const isMuted = mutedLabelSet.has(label)}
						<button
							type="button"
							class={`shrink-0 rounded-full border px-2 py-0.5 text-xs font-semibold transition focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-white/80 ${
								isSelected
									? 'border-primary-300 bg-primary-500 text-white shadow-[0_0_0_1px_rgba(255,255,255,0.18)]'
									: isMuted
										? 'border-white/10 bg-slate-800/80 text-white/55 hover:bg-slate-700/85'
										: 'border-white/10 bg-slate-900/90 text-white hover:bg-slate-800/90'
							}`}
							aria-label={`Jump to card #${label}${isSelected ? ', selected' : ''}${isMuted ? ', not voteable' : ''}`}
							aria-pressed={isSelected}
							on:click={() => jumpToCard(label)}
						>
							#{label}
						</button>
					{/each}
				</div>
			</div>
		{/if}
	</div>
{/if}

<style>
	:global(.card-number-navigator-target-highlight) {
		animation: card-number-navigator-pulse 1.6s ease-out;
	}

	@keyframes card-number-navigator-pulse {
		0% {
			box-shadow:
				0 0 0 0 rgb(250 204 21 / 0.92),
				0 0 0 0 rgb(255 255 255 / 0.4);
		}
		40% {
			box-shadow:
				0 0 0 5px rgb(250 204 21 / 0.6),
				0 0 0 10px rgb(255 255 255 / 0.22);
		}
		100% {
			box-shadow:
				0 0 0 0 rgb(250 204 21 / 0),
				0 0 0 0 rgb(255 255 255 / 0);
		}
	}
	@media (prefers-reduced-motion: reduce) {
		:global(.card-number-navigator-target-highlight) {
			animation: none;
			box-shadow:
				0 0 0 4px rgb(250 204 21 / 0.82),
				0 0 0 8px rgb(255 255 255 / 0.2);
		}
	}

	.card-number-navigator-grid {
		max-height: 5.75rem;
		scrollbar-gutter: stable;
		overscroll-behavior: contain;
	}
</style>
