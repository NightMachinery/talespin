export interface VotingLayoutView {
	images: string[];
	badgeLabels: number[];
}

function stableHashHex(value: string): string {
	let hash = 0xcbf29ce484222325n;
	const prime = 0x100000001b3n;
	const mask = 0xffffffffffffffffn;

	for (let index = 0; index < value.length; index += 1) {
		hash ^= BigInt(value.charCodeAt(index));
		hash = (hash * prime) & mask;
	}

	return hash.toString(16).padStart(16, '0');
}

export function derivePerViewerVotingLayout(
	canonicalImages: string[],
	viewerName: string,
	votingLayoutSeed: string | null | undefined,
	enabled: boolean
): VotingLayoutView {
	if (!enabled || canonicalImages.length <= 1 || viewerName.trim() === '' || !votingLayoutSeed) {
		return {
			images: [...canonicalImages],
			badgeLabels: canonicalImages.map((_, index) => index + 1)
		};
	}

	const orderedEntries = canonicalImages
		.map((image, originalIndex) => ({
			image,
			originalIndex,
			badgeLabel: originalIndex + 1,
			sortKey: stableHashHex(`${votingLayoutSeed}:${viewerName}:${originalIndex}:${image}`)
		}))
		.sort(
			(left, right) =>
				left.sortKey.localeCompare(right.sortKey) || left.originalIndex - right.originalIndex
		);

	return {
		images: orderedEntries.map((entry) => entry.image),
		badgeLabels: orderedEntries.map((entry) => entry.badgeLabel)
	};
}
