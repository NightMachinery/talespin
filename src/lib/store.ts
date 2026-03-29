import { browser } from '$app/environment';
import { writable } from 'svelte/store';

const ROOM_ASSIGNED_NAME_KEY_PREFIX = 'room_assigned_name:';

function normalizeName(value: string): string {
	return value.trim();
}

function normalizeRoomCode(roomCode: string): string {
	return roomCode.trim().toLowerCase();
}

function roomAssignedNameKey(roomCode: string): string {
	return `${ROOM_ASSIGNED_NAME_KEY_PREFIX}${normalizeRoomCode(roomCode)}`;
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

export function getAssignedRoomName(roomCode: string, token: string): string | null {
	if (!browser || !token) {
		return null;
	}

	const raw = window.localStorage.getItem(roomAssignedNameKey(roomCode));
	if (!raw) {
		return null;
	}

	try {
		const parsed = JSON.parse(raw) as {
			token?: string;
			name?: string;
		};
		if (parsed.token !== token) {
			return null;
		}

		const normalizedName = normalizeName(parsed.name || '');
		return normalizedName === '' ? null : normalizedName;
	} catch {
		window.localStorage.removeItem(roomAssignedNameKey(roomCode));
		return null;
	}
}

export function getJoinNameForRoom(roomCode: string, preferredName: string, token: string): string {
	return getAssignedRoomName(roomCode, token) || normalizeName(preferredName);
}

export function setAssignedRoomName(roomCode: string, token: string, name: string) {
	if (!browser) {
		return;
	}

	const normalizedName = normalizeName(name);
	if (!token || normalizedName === '') {
		window.localStorage.removeItem(roomAssignedNameKey(roomCode));
		return;
	}

	window.localStorage.setItem(
		roomAssignedNameKey(roomCode),
		JSON.stringify({
			token,
			name: normalizedName
		})
	);
}

export function clearAssignedRoomName(roomCode: string) {
	if (!browser) {
		return;
	}

	window.localStorage.removeItem(roomAssignedNameKey(roomCode));
}

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
