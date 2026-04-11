import { browser } from '$app/environment';
import { writable } from 'svelte/store';
import type { LeaderboardViewMode } from '$lib/types';

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

interface PersistedLeaderboardViewPreference {
	mode: LeaderboardViewMode;
	applied_room_default_version: number;
}

const LEADERBOARD_VIEW_PREF_PREFIX = 'leaderboard_view_mode:';
const LEGACY_LEADERBOARD_BEAUTY_PREF_PREFIX = 'leaderboard_exclude_beauty:';
const DEFAULT_LEADERBOARD_VIEW_MODE: LeaderboardViewMode = 'total';

function preferenceKey(roomCode: string) {
	return `${LEADERBOARD_VIEW_PREF_PREFIX}${roomCode}`;
}

function legacyPreferenceKey(roomCode: string) {
	return `${LEGACY_LEADERBOARD_BEAUTY_PREF_PREFIX}${roomCode}`;
}

function normalizeMode(value: unknown): LeaderboardViewMode | null {
	if (
		value === 'total' ||
		value === 'story_only' ||
		value === 'beauty_only' ||
		value === 'combined'
	) {
		return value;
	}
	if (typeof value === 'boolean') {
		return value ? 'story_only' : 'total';
	}
	return null;
}

function normalizePersistedPreference(raw: unknown): PersistedLeaderboardViewPreference | null {
	if (!raw || typeof raw !== 'object') return null;
	const parsed = raw as Partial<PersistedLeaderboardViewPreference> & {
		value?: boolean;
	};
	const mode =
		normalizeMode(parsed.mode) ?? normalizeMode(parsed.value) ?? DEFAULT_LEADERBOARD_VIEW_MODE;
	return {
		mode,
		applied_room_default_version:
			typeof parsed.applied_room_default_version === 'number'
				? parsed.applied_room_default_version
				: 0
	};
}

function readPersistedPreference(roomCode: string): PersistedLeaderboardViewPreference {
	if (!browser || roomCode === '') {
		return { mode: DEFAULT_LEADERBOARD_VIEW_MODE, applied_room_default_version: 0 };
	}

	const readKey = (key: string) => {
		const raw = window.localStorage.getItem(key);
		if (!raw) return null;
		try {
			return normalizePersistedPreference(JSON.parse(raw));
		} catch {
			return null;
		}
	};

	const current = readKey(preferenceKey(roomCode));
	if (current) {
		return current;
	}

	const legacy = readKey(legacyPreferenceKey(roomCode));
	if (legacy) {
		persistPreference(roomCode, legacy);
		window.localStorage.removeItem(legacyPreferenceKey(roomCode));
		return legacy;
	}

	return { mode: DEFAULT_LEADERBOARD_VIEW_MODE, applied_room_default_version: 0 };
}

function persistPreference(roomCode: string, preference: PersistedLeaderboardViewPreference) {
	if (!browser || roomCode === '') return;
	window.localStorage.setItem(preferenceKey(roomCode), JSON.stringify(preference));
}

export const mostBeautifulStats = writable<MostBeautifulPlayerStats[]>([]);
export const mostBeautifulStatsLoading = writable(false);
export const mostBeautifulStatsError = writable('');

export const currentRoomCode = writable('');
export const leaderboardViewMode = writable<LeaderboardViewMode>(DEFAULT_LEADERBOARD_VIEW_MODE);
export const roomLeaderboardViewModeDefault = writable<LeaderboardViewMode>(
	DEFAULT_LEADERBOARD_VIEW_MODE
);
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
	leaderboardViewMode.set(preference.mode);
}

export function setLeaderboardViewModePreference(mode: LeaderboardViewMode) {
	leaderboardViewMode.set(mode);
	if (currentRoomCodeValue === '') return;
	const existing = readPersistedPreference(currentRoomCodeValue);
	persistPreference(currentRoomCodeValue, {
		mode,
		applied_room_default_version: existing.applied_room_default_version
	});
}

export function applyRoomLeaderboardViewModeDefault(
	roomCode: string,
	mode: LeaderboardViewMode,
	version: number
) {
	roomLeaderboardViewModeDefault.set(mode);
	const existing = readPersistedPreference(roomCode);
	if (version > existing.applied_room_default_version) {
		const next = {
			mode,
			applied_room_default_version: version
		};
		persistPreference(roomCode, next);
		if (roomCode === currentRoomCodeValue) {
			leaderboardViewMode.set(mode);
		}
	} else if (roomCode === currentRoomCodeValue) {
		leaderboardViewMode.set(existing.mode);
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
	leaderboardViewMode.set(DEFAULT_LEADERBOARD_VIEW_MODE);
	roomLeaderboardViewModeDefault.set(DEFAULT_LEADERBOARD_VIEW_MODE);
	memberBeautyPoints.set({});
	storytellerLeaderboardPointChange.set({});
	beautyLeaderboardPointChange.set({});
	mostBeautifulStats.set([]);
	mostBeautifulStatsLoading.set(false);
	mostBeautifulStatsError.set('');
}
