export type VotingWrongCardDisablePresetId =
	| 'off'
	| 'shy'
	| 'mild'
	| '50-50'
	| 'one'
	| 'two'
	| 'three'
	| 'moderate'
	| 'chaotic'
	| 'custom';

export interface VotingWrongCardDisablePreset {
	id: Exclude<VotingWrongCardDisablePresetId, 'custom'>;
	label: string;
	distribution: number[];
}

const EPSILON = 1e-6;

export const DEFAULT_VOTING_WRONG_CARD_DISABLE_DISTRIBUTION = [1];
// Mirrors the backend's maximum meaningful X for a 64-member room:
// (max other guessers * max nominations per guesser) - (minimum legal votes + storyteller card).
export const MAX_VOTING_WRONG_CARD_DISABLE_X = 18 * (64 - 2) - 2;

export const VOTING_WRONG_CARD_DISABLE_PRESETS: VotingWrongCardDisablePreset[] = [
	{
		id: 'off',
		label: 'Off',
		distribution: [1]
	},
	{
		id: 'shy',
		label: 'Shy',
		distribution: [0.9, 0.1]
	},
	{
		id: 'mild',
		label: 'Mild',
		distribution: [0.7, 0.3]
	},
	{
		id: '50-50',
		label: '50-50',
		distribution: [0.5, 0.5]
	},
	{
		id: 'one',
		label: 'One',
		distribution: [0, 1]
	},
	{
		id: 'two',
		label: 'Two',
		distribution: [0, 0, 1]
	},
	{
		id: 'three',
		label: 'Three',
		distribution: [0, 0, 0, 1]
	},
	{
		id: 'moderate',
		label: 'Moderate',
		distribution: [0.3, 0.5, 0.2]
	},
	{
		id: 'chaotic',
		label: 'Chaotic',
		distribution: [0.2, 0.3, 0.3, 0.2]
	}
];

function comparableVotingWrongCardDisableDistribution(distribution: number[]): number[] {
	const comparable = [...normalizeVotingWrongCardDisableDistribution(distribution)];
	while (comparable.length > 1 && comparable[comparable.length - 1] <= EPSILON) {
		comparable.pop();
	}
	return comparable;
}

export function normalizeVotingWrongCardDisableDistribution(distribution: number[]): number[] {
	const targetLength = Math.max(1, distribution.length || 0);
	const sanitized = Array.from({ length: targetLength }, (_, index) => {
		const value = distribution[index];
		return Number.isFinite(value) && value > 0 ? value : 0;
	});
	const total = sanitized.reduce((sum, value) => sum + value, 0);
	if (total <= EPSILON) {
		return [1, ...Array(targetLength - 1).fill(0)];
	}
	return sanitized.map((value) => value / total);
}

export function resizeVotingWrongCardDisableDistribution(
	distribution: number[],
	maxX: number
): number[] {
	const normalized = normalizeVotingWrongCardDisableDistribution(distribution);
	const targetLength = Math.max(1, Math.min(MAX_VOTING_WRONG_CARD_DISABLE_X, Math.floor(maxX)) + 1);
	if (targetLength === normalized.length) {
		return normalized;
	}
	if (targetLength > normalized.length) {
		return [...normalized, ...Array(targetLength - normalized.length).fill(0)];
	}
	return normalizeVotingWrongCardDisableDistribution(normalized.slice(0, targetLength));
}

export function setVotingWrongCardDisableProbability(
	distribution: number[],
	index: number,
	probability: number
): number[] {
	const normalized = normalizeVotingWrongCardDisableDistribution(distribution);
	if (normalized.length <= 1 || index < 0 || index >= normalized.length) {
		return normalized;
	}

	const clampedProbability = Math.min(1, Math.max(0, probability));
	const otherIndices = normalized
		.map((_, currentIndex) => currentIndex)
		.filter((currentIndex) => currentIndex !== index);
	const remainingProbability = Math.max(0, 1 - clampedProbability);
	const otherTotal = otherIndices.reduce((sum, currentIndex) => sum + normalized[currentIndex], 0);
	const next = [...normalized];
	next[index] = clampedProbability;

	if (otherTotal <= EPSILON) {
		const fallback = otherIndices.length > 0 ? remainingProbability / otherIndices.length : 0;
		for (const currentIndex of otherIndices) {
			next[currentIndex] = fallback;
		}
	} else {
		const scale = remainingProbability / otherTotal;
		for (const currentIndex of otherIndices) {
			next[currentIndex] = normalized[currentIndex] * scale;
		}
	}

	return normalizeVotingWrongCardDisableDistribution(next);
}

export function areVotingWrongCardDisableDistributionsEqual(
	a: number[],
	b: number[],
	tolerance = EPSILON
): boolean {
	const normalizedA = normalizeVotingWrongCardDisableDistribution(a);
	const normalizedB = normalizeVotingWrongCardDisableDistribution(b);
	if (normalizedA.length !== normalizedB.length) {
		return false;
	}
	return normalizedA.every((value, index) => Math.abs(value - normalizedB[index]) <= tolerance);
}

export function findVotingWrongCardDisablePresetId(
	distribution: number[]
): VotingWrongCardDisablePresetId {
	const comparableDistribution = comparableVotingWrongCardDisableDistribution(distribution);
	for (const preset of VOTING_WRONG_CARD_DISABLE_PRESETS) {
		if (
			areVotingWrongCardDisableDistributionsEqual(
				comparableVotingWrongCardDisableDistribution(preset.distribution),
				comparableDistribution
			)
		) {
			return preset.id;
		}
	}
	return 'custom';
}

export function getVotingWrongCardDisablePreset(
	id: Exclude<VotingWrongCardDisablePresetId, 'custom'>
): VotingWrongCardDisablePreset | undefined {
	return VOTING_WRONG_CARD_DISABLE_PRESETS.find((preset) => preset.id === id);
}
