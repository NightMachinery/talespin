import { http_host } from '$lib/gameServer';

export type CardSourceInfo = {
	relative_source_path: string;
};

export function cardImageUrl(card: string) {
	return `${http_host}/cards/${card}`;
}

export function originalCardImageUrl(card: string) {
	return `${http_host}/cards/${card}_original`;
}

export function cardSourceInfoUrl(card: string) {
	return `${http_host}/cards/${card}/source-info`;
}

export function originalCardDownloadFilename(card: string) {
	return `${card}_original`;
}

export async function fetchCardSourceInfo(card: string): Promise<CardSourceInfo | null> {
	try {
		const response = await fetch(cardSourceInfoUrl(card));
		if (!response.ok) return null;
		const payload = (await response.json()) as Partial<CardSourceInfo>;
		if (typeof payload.relative_source_path !== 'string' || payload.relative_source_path === '') {
			return null;
		}
		return { relative_source_path: payload.relative_source_path };
	} catch {
		return null;
	}
}
