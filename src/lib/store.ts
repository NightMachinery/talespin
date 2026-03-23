import { browser } from '$app/environment';
import { writable } from 'svelte/store';

function normalizeName(value: string): string {
	return value.trim();
}

function createNormalizedNameStore() {
	const initialValue = browser ? normalizeName(window.localStorage.getItem('name') || '') : '';
	const { subscribe, set: setRaw, update: updateRaw } = writable(initialValue);

	return {
		subscribe,
		set(value: string) {
			setRaw(normalizeName(value));
		},
		update(updater: (value: string) => string) {
			updateRaw((value) => normalizeName(updater(value)));
		}
	};
}

export const nameStore = createNormalizedNameStore();
const existingToken = browser ? window.localStorage.getItem('player_token') || '' : '';
const generatedToken = browser && existingToken === '' ? window.crypto.randomUUID() : existingToken;
export const playerTokenStore = writable(browser ? generatedToken : '');

nameStore.subscribe((value) => {
	if (browser) {
		window.localStorage.setItem('name', value);
	}
});

playerTokenStore.subscribe((value) => {
	if (browser) {
		if (value) {
			window.localStorage.setItem('player_token', value);
		} else {
			window.localStorage.removeItem('player_token');
		}
	}
});
