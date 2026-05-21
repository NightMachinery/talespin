<script lang="ts">
	import { getToastStore } from '@skeletonlabs/skeleton';
	import {
		buildMigrateDeviceLink,
		copyTextToClipboard,
		currentRoomMigration
	} from '$lib/deviceMigration';

	export let fullWidth = false;
	export let className = '';

	const toastStore = getToastStore();

	$: migrateLink =
		typeof window === 'undefined'
			? ''
			: buildMigrateDeviceLink({
					origin: window.location.origin,
					roomCode: $currentRoomMigration.roomCode,
					roomAuthId: $currentRoomMigration.roomAuthId,
					roomPassword: $currentRoomMigration.roomPassword
				});
	$: isDisabled = migrateLink === '';
	$: buttonClasses = ['btn', 'variant-filled', fullWidth ? 'w-full' : '', className]
		.filter(Boolean)
		.join(' ');

	async function copyMigrateLink() {
		if (migrateLink === '') return;
		const copied = await copyTextToClipboard(migrateLink);
		toastStore.trigger({
			message: copied ? '📱 Device migration link copied' : 'Could not copy device migration link',
			autohide: true,
			timeout: 2500
		});
	}
</script>

<button
	class={buttonClasses}
	disabled={isDisabled}
	title="Copy a room-specific link to continue this room identity on another device."
	on:click={copyMigrateLink}>Migrate Device</button
>
