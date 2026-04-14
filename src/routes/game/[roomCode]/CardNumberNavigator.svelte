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
	export let title = 'Jump to card';
	export let collapsedLabel = 'Card navigator';

	const TARGET_HIGHLIGHT_CLASS = 'card-number-navigator-target-highlight';
	let highlightTimeout: number | undefined;

	$: uniqueOrderedLabels = [...new Set(cardNumberLabels)]
		.filter((label): label is number => Number.isFinite(label) && label > 0)
		.sort((a, b) => a - b);
	$: selectedLabelSet = new Set(selectedLabels);
	$: mutedLabelSet = new Set(mutedLabels);
	$: shouldShowNavigator = $stickyVotingCardNavigatorEnabled && uniqueOrderedLabels.length > 1;

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
	<div class="sticky top-2 z-10 mb-3">
		{#if $stickyVotingCardNavigatorCollapsed}
			<div class="flex justify-start">
				<button
					type="button"
					class="rounded-full border border-white/15 bg-slate-950/85 px-3 py-1.5 text-xs font-semibold text-white shadow-lg backdrop-blur transition hover:bg-slate-900/90 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-white/80"
					on:click={() => stickyVotingCardNavigatorCollapsed.set(false)}
				>
					Show {collapsedLabel}
				</button>
			</div>
		{:else}
			<div
				class="rounded-2xl border border-white/10 bg-slate-950/80 px-3 py-2 shadow-lg backdrop-blur"
			>
				<div class="mb-2 flex items-center justify-between gap-3">
					<p class="text-xs font-semibold uppercase tracking-[0.18em] text-white/70">{title}</p>
					<button
						type="button"
						class="rounded-full border border-white/15 px-2.5 py-1 text-xs font-medium text-white/80 transition hover:bg-white/10 hover:text-white focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-white/80"
						on:click={() => stickyVotingCardNavigatorCollapsed.set(true)}
					>
						Collapse
					</button>
				</div>
				<div class="-mx-1 flex gap-2 overflow-x-auto px-1 pb-1">
					{#each uniqueOrderedLabels as label}
						{@const isSelected = selectedLabelSet.has(label)}
						{@const isMuted = mutedLabelSet.has(label)}
						<button
							type="button"
							class={`shrink-0 rounded-full border px-3 py-1 text-sm font-semibold transition focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-white/80 ${
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
</style>
