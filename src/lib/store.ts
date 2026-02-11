import { browser } from '$app/environment';
import { writable } from 'svelte/store';

export const nameStore = writable(browser ? window.localStorage.getItem('name') || '' : '');
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
