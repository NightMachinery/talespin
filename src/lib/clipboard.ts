import { browser } from '$app/environment';

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
