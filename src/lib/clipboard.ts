import { browser } from '$app/environment';

export async function copyTextToClipboard(text: string) {
	if (!browser || text.trim() === '') {
		return false;
	}

	try {
		await navigator.clipboard.writeText(text);
		return true;
	} catch {
		return copyTextWithLegacyCommand(text);
	}
}

function copyTextWithLegacyCommand(text: string) {
	try {
		const textArea = document.createElement('textarea');
		textArea.value = text;
		textArea.style.position = 'fixed';
		textArea.style.opacity = '0';
		document.body.appendChild(textArea);
		textArea.focus();
		textArea.select();
		const copied = document.execCommand('copy');
		document.body.removeChild(textArea);
		return copied;
	} catch {
		return false;
	}
}
