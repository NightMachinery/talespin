export type StellaCardAnnotation = {
	label: string;
	className: string;
};

export type StellaCardEffectPresentation = {
	highlightClass: string;
	annotation?: StellaCardAnnotation;
};

export function getStellaCardEffectPresentation(points: number): StellaCardEffectPresentation {
	if (points >= 3) {
		return {
			highlightClass: 'brightness-110 ring-4 ring-cyan-300 stella-card-effect-super-spark',
			annotation: {
				label: 'Super-Spark',
				className: 'stella-card-effect-badge stella-card-effect-badge-super-spark'
			}
		};
	}

	if (points >= 2) {
		return {
			highlightClass: 'brightness-105 ring-4 ring-success-500 stella-card-effect-spark',
			annotation: {
				label: 'Spark',
				className: 'stella-card-effect-badge stella-card-effect-badge-spark'
			}
		};
	}

	return {
		highlightClass: 'brightness-95 ring-4 ring-error-500 stella-card-effect-fall',
		annotation: {
			label: 'Fell',
			className: 'stella-card-effect-badge stella-card-effect-badge-fall'
		}
	};
}
