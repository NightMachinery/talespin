import { writable } from 'svelte/store';

export const clueRatingEnabled = writable(false);
export const clueRatingMaxStars = writable(3);
export const clueRatingMaxStarsMin = writable(1);
export const clueRatingMaxStarsMax = writable(10);
export const clueRatingTimerEnabled = writable(true);
export const clueRatingTimerDurationS = writable(20);
export const forceClueRatingTimer = writable(false);
export const memberClueRatingAverage = writable<Record<string, number>>({});
export const memberClueRatingRounds = writable<Record<string, number>>({});
export const resultsPlayerClueRatings = writable<Record<string, number>>({});
export const resultsClueRatingAverage = writable<number | null>(null);
export const resultsClueRatingCount = writable(0);
export const resultsClueRatingBonus = writable(0);

export function setClueRatingRoomState(payload: {
	enabled?: boolean;
	maxStars?: number;
	minStars?: number;
	maxStarsLimit?: number;
	timerEnabled?: boolean;
	timerDurationS?: number;
	forceTimer?: boolean;
	memberAverage?: Record<string, number>;
	memberRounds?: Record<string, number>;
}) {
	clueRatingEnabled.set(payload.enabled ?? false);
	clueRatingMaxStars.set(payload.maxStars ?? 3);
	clueRatingMaxStarsMin.set(payload.minStars ?? 1);
	clueRatingMaxStarsMax.set(payload.maxStarsLimit ?? 10);
	clueRatingTimerEnabled.set(payload.timerEnabled ?? true);
	clueRatingTimerDurationS.set(payload.timerDurationS ?? 20);
	forceClueRatingTimer.set(payload.forceTimer ?? false);
	memberClueRatingAverage.set(payload.memberAverage ?? {});
	memberClueRatingRounds.set(payload.memberRounds ?? {});
}

export function setClueRatingResults(payload: {
	playerRatings?: Record<string, number>;
	average?: number | null;
	count?: number;
	bonus?: number;
}) {
	resultsPlayerClueRatings.set(payload.playerRatings ?? {});
	resultsClueRatingAverage.set(payload.average ?? null);
	resultsClueRatingCount.set(payload.count ?? 0);
	resultsClueRatingBonus.set(payload.bonus ?? 0);
}

export function clearClueRatingResults() {
	resultsPlayerClueRatings.set({});
	resultsClueRatingAverage.set(null);
	resultsClueRatingCount.set(0);
	resultsClueRatingBonus.set(0);
}

export function resetClueRatingClientState() {
	setClueRatingRoomState({});
	clearClueRatingResults();
}
