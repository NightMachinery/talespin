export interface StellaWordPackPreset {
	name: string;
	words: string;
}

export interface StellaWordPackOption extends StellaWordPackPreset {
	key: string;
	builtin: boolean;
	unsaved: boolean;
}

export const LOCAL_STELLA_PACKS_KEY = 'stella_word_pack_presets';
export const LOCAL_PRESET_KEY_PREFIX = 'local:';
export const BUILTIN_PRESET_KEY_PREFIX = 'builtin:';
export const UNSAVED_PRESET_KEY = 'current:unsaved';
export const UNSAVED_PRESET_NAME = 'Unsaved Pack';

export function normalizeWordPackText(rawWords: string) {
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

function sortWordPackPresets<T extends StellaWordPackPreset>(presets: T[]) {
	return [...presets].sort((a, b) => a.name.localeCompare(b.name));
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

export function parseSavedStellaWordPackPresets(rawPresets: string | null): StellaWordPackPreset[] {
	try {
		const parsed = JSON.parse(rawPresets || '[]');
		if (!Array.isArray(parsed)) {
			return [];
		}
		return sortWordPackPresets(
			parsed
				.filter(
					(entry): entry is StellaWordPackPreset =>
						typeof entry?.name === 'string' && typeof entry?.words === 'string'
				)
				.map((preset) => ({
					name: preset.name.trim(),
					words: normalizeWordPackText(preset.words)
				}))
				.filter((preset) => preset.name.length > 0 && preset.words.length > 0)
		);
	} catch {
		return [];
	}
}

export function buildStellaWordPackOptions(
	savedPresets: StellaWordPackPreset[],
	activeWords: string
): StellaWordPackOption[] {
	const options: StellaWordPackOption[] = [
		...BUILTIN_STELLA_WORD_PACK_PRESETS.map((preset) => ({
			...preset,
			key: `${BUILTIN_PRESET_KEY_PREFIX}${preset.name}`,
			builtin: true,
			unsaved: false
		})),
		...sortWordPackPresets(savedPresets).map((preset) => ({
			...preset,
			key: `${LOCAL_PRESET_KEY_PREFIX}${preset.name}`,
			builtin: false,
			unsaved: false
		}))
	];
	const normalizedActiveWords = normalizeWordPackText(activeWords);
	if (
		normalizedActiveWords &&
		!options.some((option) => normalizeWordPackText(option.words) === normalizedActiveWords)
	) {
		options.push({
			name: UNSAVED_PRESET_NAME,
			words: normalizedActiveWords,
			key: UNSAVED_PRESET_KEY,
			builtin: false,
			unsaved: true
		});
	}
	return sortWordPackPresets(options);
}

export function findStellaWordPackOptionByWords(
	options: StellaWordPackOption[],
	words: string
): StellaWordPackOption | null {
	const normalizedWords = normalizeWordPackText(words);
	if (!normalizedWords) return null;
	return options.find((option) => normalizeWordPackText(option.words) === normalizedWords) ?? null;
}

export function formatStellaWordPackOptionLabel(option: StellaWordPackOption) {
	if (option.unsaved) {
		return option.name;
	}
	return option.builtin ? `${option.name} (built-in)` : option.name;
}
