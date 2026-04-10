import { browser } from '$app/environment';
import { writable } from 'svelte/store';

export interface MostBeautifulVoterStats {
	player_hash: string;
	display_name: string;
	votes: number;
}

export interface MostBeautifulPlayerStats {
	player_hash: string;
	display_name: string;
	votes_received: number;
	rounds_won: number;
	tie_round_wins: number;
	decisive_round_wins: number;
	voters: MostBeautifulVoterStats[];
}

interface MostBeautifulStatsResponse {
	players: MostBeautifulPlayerStats[];
}

interface PersistedLeaderboardBeautyPreference {
	value: boolean;
	applied_room_default_version: number;
}

const LEADERBOARD_BEAUTY_PREF_PREFIX = 'leaderboard_exclude_beauty:';

function preferenceKey(roomCode: string) {
	return `${LEADERBOARD_BEAUTY_PREF_PREFIX}${roomCode}`;
}

function readPersistedPreference(roomCode: string): PersistedLeaderboardBeautyPreference {
	if (!browser || roomCode === '') {
		return { value: false, applied_room_default_version: 0 };
	}
	try {
		const raw = window.localStorage.getItem(preferenceKey(roomCode));
		if (!raw) {
			return { value: false, applied_room_default_version: 0 };
		}
		const parsed = JSON.parse(raw) as Partial<PersistedLeaderboardBeautyPreference>;
		return {
			value: parsed.value === true,
			applied_room_default_version:
				typeof parsed.applied_room_default_version === 'number'
					? parsed.applied_room_default_version
					: 0
		};
	} catch {
		return { value: false, applied_room_default_version: 0 };
	}
}

function persistPreference(roomCode: string, preference: PersistedLeaderboardBeautyPreference) {
	if (!browser || roomCode === '') return;
	window.localStorage.setItem(preferenceKey(roomCode), JSON.stringify(preference));
}

export const mostBeautifulStats = writable<MostBeautifulPlayerStats[]>([]);
export const mostBeautifulStatsLoading = writable(false);
export const mostBeautifulStatsError = writable('');

export const currentRoomCode = writable('');
export const leaderboardExcludeBeauty = writable(false);
export const roomLeaderboardExcludeBeautyDefault = writable(false);
export const memberBeautyPoints = writable<Record<string, number>>({});
export const storytellerLeaderboardPointChange = writable<Record<string, number>>({});
export const beautyLeaderboardPointChange = writable<Record<string, number>>({});

let currentRoomCodeValue = '';
currentRoomCode.subscribe((roomCode) => {
	currentRoomCodeValue = roomCode;
});

export function setMostBeautifulRoom(roomCode: string) {
	currentRoomCode.set(roomCode);
	const preference = readPersistedPreference(roomCode);
	leaderboardExcludeBeauty.set(preference.value);
}

export function setLeaderboardExcludeBeautyPreference(value: boolean) {
	leaderboardExcludeBeauty.set(value);
	if (currentRoomCodeValue === '') return;
	const existing = readPersistedPreference(currentRoomCodeValue);
	persistPreference(currentRoomCodeValue, {
		value,
		applied_room_default_version: existing.applied_room_default_version
	});
}

export function applyRoomLeaderboardExcludeBeautyDefault(
	roomCode: string,
	value: boolean,
	version: number
) {
	roomLeaderboardExcludeBeautyDefault.set(value);
	const existing = readPersistedPreference(roomCode);
	if (version > existing.applied_room_default_version) {
		const next = {
			value,
			applied_room_default_version: version
		};
		persistPreference(roomCode, next);
		if (roomCode === currentRoomCodeValue) {
			leaderboardExcludeBeauty.set(value);
		}
	} else if (roomCode === currentRoomCodeValue) {
		leaderboardExcludeBeauty.set(existing.value);
	}
}

export async function refreshMostBeautifulStats() {
	mostBeautifulStatsLoading.set(true);
	mostBeautifulStatsError.set('');
	try {
		const response = await fetch('/most-beautiful-stats', {
			method: 'GET',
			headers: {
				'Cache-Control': 'no-cache'
			}
		});
		if (!response.ok) {
			throw new Error(`Failed with ${response.status}`);
		}
		const payload = (await response.json()) as MostBeautifulStatsResponse;
		mostBeautifulStats.set(Array.isArray(payload.players) ? payload.players : []);
	} catch (error) {
		console.error('Failed to refresh Most Beautiful stats', error);
		mostBeautifulStatsError.set('Failed to load Most Beautiful stats.');
	} finally {
		mostBeautifulStatsLoading.set(false);
	}
}

export function resetMostBeautifulClientState() {
	currentRoomCode.set('');
	leaderboardExcludeBeauty.set(false);
	roomLeaderboardExcludeBeautyDefault.set(false);
	memberBeautyPoints.set({});
	storytellerLeaderboardPointChange.set({});
	beautyLeaderboardPointChange.set({});
	mostBeautifulStats.set([]);
	mostBeautifulStatsLoading.set(false);
	mostBeautifulStatsError.set('');
}
