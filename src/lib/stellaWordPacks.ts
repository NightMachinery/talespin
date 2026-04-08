export interface StellaWordPackPreset {
	name: string;
	words: string;
}

function normalizeWordPackText(rawWords: string) {
	return rawWords
		.split(/\r?\n/u)
		.map((word) => word.trim())
		.filter((word) => word.length > 0)
		.join('\n');
}

function presetNameFromPath(path: string) {
	return (
		path
			.split('/')
			.pop()
			?.replace(/\.txt$/iu, '') ?? path
	);
}

const rawWordPackModules = import.meta.glob('../../wordpacks/*.txt', {
	eager: true,
	import: 'default',
	query: '?raw'
}) as Record<string, string>;

export const BUILTIN_STELLA_WORD_PACK_PRESETS: StellaWordPackPreset[] = Object.entries(
	rawWordPackModules
)
	.map(([path, rawWords]) => ({
		name: presetNameFromPath(path),
		words: normalizeWordPackText(rawWords)
	}))
	.filter((preset) => preset.words.length > 0)
	.sort((a, b) => a.name.localeCompare(b.name));
