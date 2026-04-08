import { browser } from '$app/environment';
import { writable } from 'svelte/store';

function createPersistentBooleanStore(key: string, defaultValue: boolean) {
	const storedValue = browser ? window.localStorage.getItem(key) : null;
	const initialValue = storedValue === null ? defaultValue : storedValue === 'true' ? true : false;
	const { subscribe, set: setRaw, update: updateRaw } = writable(initialValue);

	function persist(value: boolean) {
		if (!browser) return;
		window.localStorage.setItem(key, value ? 'true' : 'false');
	}

	return {
		subscribe,
		set(value: boolean) {
			setRaw(value);
			persist(value);
		},
		update(updater: (value: boolean) => boolean) {
			updateRaw((value) => {
				const nextValue = updater(value);
				persist(nextValue);
				return nextValue;
			});
		}
	};
}

// Default to scrollable cards for consistent gameplay presentation.
export const cardsFitToHeight = writable(false);
export const stageChangeSoundCuesEnabled = createPersistentBooleanStore(
	'stage_change_sound_cues_enabled',
	true
	// false
);
export const transparentCardNameOverlays = createPersistentBooleanStore(
	'transparent_card_name_overlays',
	false
);
export const hideNonSelectedStellaRevealCards = createPersistentBooleanStore(
	'hide_non_selected_stella_reveal_cards',
	true
);
