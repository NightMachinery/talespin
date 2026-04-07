import type { WinCondition } from '$lib/types';

export function formatWinCondition(winCondition: WinCondition): string {
	if (winCondition.mode === 'points') {
		return `First to ${winCondition.target_points} point${winCondition.target_points === 1 ? '' : 's'}!`;
	}
	if (winCondition.mode === 'cycles') {
		return `${winCondition.target_cycles} full storyteller cycle${winCondition.target_cycles === 1 ? '' : 's'}`;
	}
	if (winCondition.mode === 'fixed_rounds') {
		return `${winCondition.target_rounds} fixed round${winCondition.target_rounds === 1 ? '' : 's'}`;
	}
	return 'Play until cards finish';
}
