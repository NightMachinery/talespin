import { browser } from '$app/environment';
import { writable } from 'svelte/store';

export interface RoomMigrationState {
	roomCode: string;
	roomAuthId: string;
	roomPassword: string;
}

const INITIAL_ROOM_MIGRATION_STATE: RoomMigrationState = {
	roomCode: '',
	roomAuthId: '',
	roomPassword: ''
};

export const currentRoomMigration = writable<RoomMigrationState>(INITIAL_ROOM_MIGRATION_STATE);

function normalizeRoomCode(roomCode: string): string {
	return roomCode.trim().toLowerCase();
}

function normalizeRoomAuthId(roomAuthId: string): string {
	return roomAuthId.trim();
}

function normalizeRoomPassword(roomPassword: string): string {
	return roomPassword.trim();
}

export function readRoomMigrationOverride(url: URL) {
	const roomAuthId = normalizeRoomAuthId(url.searchParams.get('room_auth_id') || '');
	const roomPassword = normalizeRoomPassword(url.searchParams.get('room_password') || '');

	return {
		roomAuthId,
		roomPassword,
		active: roomAuthId !== ''
	};
}

export function setCurrentRoomMigrationRoom(roomCode: string) {
	currentRoomMigration.update((state) => ({
		...state,
		roomCode: normalizeRoomCode(roomCode)
	}));
}

export function setCurrentRoomAuthId(roomAuthId: string) {
	currentRoomMigration.update((state) => ({
		...state,
		roomAuthId: normalizeRoomAuthId(roomAuthId)
	}));
}

export function setCurrentRoomPassword(roomPassword: string) {
	currentRoomMigration.update((state) => ({
		...state,
		roomPassword: normalizeRoomPassword(roomPassword)
	}));
}

export function resetCurrentRoomMigration() {
	currentRoomMigration.set(INITIAL_ROOM_MIGRATION_STATE);
}

export function buildMigrateDeviceLink({
	origin,
	roomCode,
	roomAuthId,
	roomPassword
}: {
	origin: string;
	roomCode: string;
	roomAuthId: string;
	roomPassword?: string;
}) {
	const normalizedRoomCode = normalizeRoomCode(roomCode);
	const normalizedRoomAuthId = normalizeRoomAuthId(roomAuthId);
	if (!origin || normalizedRoomCode === '' || normalizedRoomAuthId === '') {
		return '';
	}

	const migrateUrl = new URL(`/game/${normalizedRoomCode}`, origin);
	migrateUrl.searchParams.set('room_auth_id', normalizedRoomAuthId);

	const normalizedRoomPassword = normalizeRoomPassword(roomPassword || '');
	if (normalizedRoomPassword !== '') {
		migrateUrl.searchParams.set('room_password', normalizedRoomPassword);
	}

	return migrateUrl.toString();
}

export async function copyTextToClipboard(text: string) {
	if (!browser || text.trim() === '') {
		return;
	}

	try {
		await navigator.clipboard.writeText(text);
	} catch {
		const textArea = document.createElement('textarea');
		textArea.value = text;
		textArea.style.position = 'fixed';
		textArea.style.opacity = '0';
		document.body.appendChild(textArea);
		textArea.focus();
		textArea.select();
		document.execCommand('copy');
		document.body.removeChild(textArea);
	}
}
