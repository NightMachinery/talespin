<script lang="ts">
	import { createEventDispatcher } from 'svelte';
	import CopyIcon from 'svelte-feather-icons/src/icons/CopyIcon.svelte';
	import DownloadIcon from 'svelte-feather-icons/src/icons/DownloadIcon.svelte';
	import ExternalLinkIcon from 'svelte-feather-icons/src/icons/ExternalLinkIcon.svelte';
	import FileTextIcon from 'svelte-feather-icons/src/icons/FileTextIcon.svelte';
	import LinkIcon from 'svelte-feather-icons/src/icons/LinkIcon.svelte';
	import XIcon from 'svelte-feather-icons/src/icons/XIcon.svelte';
	import { getToastStore } from '@skeletonlabs/skeleton';

	import {
		cardImageUrl,
		fetchCardSourceInfo,
		originalCardDownloadFilename,
		originalCardImageUrl,
		type CardSourceInfo
	} from '$lib/cardImageUrls';
	import { CARD_IMAGE_ALT_TEXT } from '$lib/cardImageText';
	import { copyTextToClipboard } from '$lib/clipboard';

	export let card = '';

	const dispatch = createEventDispatcher<{ close: void }>();
	const toastStore = getToastStore();

	let sourceInfo: CardSourceInfo | null = null;
	let sourceInfoCard = '';

	$: originalUrl = card ? originalCardImageUrl(card) : '';
	$: compressedUrl = card ? cardImageUrl(card) : '';

	$: if (card && card !== sourceInfoCard) {
		sourceInfoCard = card;
		sourceInfo = null;
		void fetchCardSourceInfo(card).then((info) => {
			if (sourceInfoCard === card) {
				sourceInfo = info;
			}
		});
	}

	function close() {
		dispatch('close');
	}

	function handleKeydown(event: KeyboardEvent) {
		if (event.key === 'Escape') {
			event.preventDefault();
			close();
		}
	}

	function copyValue(value: string, successMessage: string) {
		void copyTextToClipboard(value).then((copied) => {
			toastStore.trigger({
				message: copied ? successMessage : 'Could not copy',
				autohide: true,
				timeout: 1800
			});
		});
	}

	function downloadOriginal() {
		if (!originalUrl) return;
		const link = document.createElement('a');
		link.href = originalUrl;
		link.download = originalCardDownloadFilename(card);
		document.body.appendChild(link);
		link.click();
		link.remove();
	}

	function openOriginal() {
		if (!originalUrl) return;
		window.open(originalUrl, '_blank', 'noopener,noreferrer');
	}
</script>

<svelte:window on:keydown={handleKeydown} />

{#if card}
	<div
		class="fixed inset-0 z-[100] flex items-center justify-center p-3"
		role="dialog"
		aria-modal="true"
		aria-label="Card image preview"
	>
		<button
			type="button"
			class="absolute inset-0 bg-black/90 backdrop-blur-sm"
			aria-label="Close image preview"
			on:click={close}
		/>
		<div class="relative flex h-full w-full max-w-6xl flex-col gap-3" role="document">
			<div class="flex flex-wrap items-center justify-end gap-2">
				<button
					type="button"
					class="btn variant-filled px-3 py-2 text-sm"
					title="Copy original image URL"
					on:click={() => copyValue(originalUrl, 'Original image URL copied')}
				>
					<CopyIcon size="18" />
					<span>Copy URL</span>
				</button>
				<button
					type="button"
					class="btn variant-filled px-3 py-2 text-sm"
					title="Download original image"
					on:click={downloadOriginal}
				>
					<DownloadIcon size="18" />
					<span>Download</span>
				</button>
				<button
					type="button"
					class="btn variant-filled px-3 py-2 text-sm"
					title="Open original image in new tab"
					on:click={openOriginal}
				>
					<ExternalLinkIcon size="18" />
					<span>Open</span>
				</button>
				{#if sourceInfo}
					<button
						type="button"
						class="btn variant-ghost-surface px-3 py-2 text-sm"
						title="Copy compressed card URL"
						on:click={() => copyValue(compressedUrl, 'Compressed image URL copied')}
					>
						<LinkIcon size="18" />
						<span>Compressed URL</span>
					</button>
					<button
						type="button"
						class="btn variant-ghost-surface px-3 py-2 text-sm"
						title="Copy source relative path"
						on:click={() => copyValue(sourceInfo?.relative_source_path ?? '', 'Source path copied')}
					>
						<FileTextIcon size="18" />
						<span>Source Path</span>
					</button>
				{/if}
				<button
					type="button"
					class="btn variant-filled-error px-3 py-2 text-sm"
					title="Close preview"
					on:click={close}
				>
					<XIcon size="18" />
					<span>Close</span>
				</button>
			</div>

			<div class="min-h-0 flex-1 overflow-hidden rounded bg-black/40">
				<img
					class="h-full w-full object-contain"
					src={originalUrl}
					alt={CARD_IMAGE_ALT_TEXT}
					draggable="false"
				/>
			</div>
		</div>
	</div>
{/if}
