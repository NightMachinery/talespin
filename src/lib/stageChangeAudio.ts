import { browser } from '$app/environment';

export type StageCueStage =
	| 'ActiveChooses'
	| 'PlayersChoose'
	| 'Voting'
	| 'Results'
	| 'StellaAssociate'
	| 'StellaReveal'
	| 'StellaResults';

type CueNote = {
	frequencyHz: number;
	durationS: number;
};

type BrowserWindow = Window &
	typeof globalThis & {
		webkitAudioContext?: typeof AudioContext;
	};

const CUE_NOTE_GAP_S = 0.03;
const CUE_VOLUME = 0.045;
const CUE_FADE_IN_S = 0.015;
const CUE_FADE_OUT_S = 0.04;
const STAGE_CUES: Record<StageCueStage, CueNote[]> = {
	ActiveChooses: [
		{ frequencyHz: 523.25, durationS: 0.18 },
		{ frequencyHz: 659.25, durationS: 0.18 }
	],
	PlayersChoose: [
		{ frequencyHz: 587.33, durationS: 0.18 },
		{ frequencyHz: 698.46, durationS: 0.18 }
	],
	Voting: [
		{ frequencyHz: 783.99, durationS: 0.18 },
		{ frequencyHz: 987.77, durationS: 0.18 }
	],
	Results: [
		{ frequencyHz: 987.77, durationS: 0.16 },
		{ frequencyHz: 783.99, durationS: 0.16 },
		{ frequencyHz: 659.25, durationS: 0.2 }
	],
	StellaAssociate: [
		{ frequencyHz: 440, durationS: 0.14 },
		{ frequencyHz: 554.37, durationS: 0.16 },
		{ frequencyHz: 659.25, durationS: 0.18 }
	],
	StellaReveal: [
		{ frequencyHz: 659.25, durationS: 0.15 },
		{ frequencyHz: 783.99, durationS: 0.15 },
		{ frequencyHz: 880, durationS: 0.2 }
	],
	StellaResults: [
		{ frequencyHz: 880, durationS: 0.16 },
		{ frequencyHz: 1174.66, durationS: 0.18 },
		{ frequencyHz: 987.77, durationS: 0.22 }
	]
};

const SCOUT_TURN_CUE: CueNote[] = [
	{ frequencyHz: 740, durationS: 0.1 },
	{ frequencyHz: 987.77, durationS: 0.16 },
	{ frequencyHz: 1318.51, durationS: 0.2 }
];

let audioContext: AudioContext | null = null;

function getAudioContextCtor(): typeof AudioContext | null {
	if (!browser || typeof window === 'undefined') return null;
	return window.AudioContext || (window as BrowserWindow).webkitAudioContext || null;
}

function getAudioContext(): AudioContext | null {
	const AudioContextCtor = getAudioContextCtor();
	if (!AudioContextCtor) return null;
	if (!audioContext) {
		audioContext = new AudioContextCtor();
	}
	return audioContext;
}

export function isStageChangeAudioSupported() {
	return getAudioContextCtor() !== null;
}

export async function unlockStageChangeAudio() {
	const context = getAudioContext();
	if (!context) return;
	if (context.state === 'suspended') {
		try {
			await context.resume();
		} catch {
			// Ignore autoplay-policy failures and keep gameplay uninterrupted.
		}
	}
}

export async function playStageChangeCue(stage: StageCueStage) {
	const context = getAudioContext();
	if (!context) return;

	if (context.state === 'suspended') {
		try {
			await context.resume();
		} catch {
			return;
		}
	}

	if (context.state !== 'running') return;

	const notes = STAGE_CUES[stage];
	let startTime = context.currentTime + 0.01;

	playCueNotes(context, notes, stage === 'Results' || stage === 'StellaResults' ? 'triangle' : 'sine', startTime);
}

export async function playScoutTurnCue() {
	const context = getAudioContext();
	if (!context) return;

	if (context.state === 'suspended') {
		try {
			await context.resume();
		} catch {
			return;
		}
	}

	if (context.state !== 'running') return;

	let startTime = context.currentTime + 0.01;
	playCueNotes(context, SCOUT_TURN_CUE, 'triangle', startTime);
}

function playCueNotes(
	context: AudioContext,
	notes: CueNote[],
	oscillatorType: OscillatorType,
	startTime: number
) {
	for (const note of notes) {
		const oscillator = context.createOscillator();
		const gainNode = context.createGain();
		const noteEndTime = startTime + note.durationS;
		const fadeOutStartTime = Math.max(startTime, noteEndTime - CUE_FADE_OUT_S);

		oscillator.type = oscillatorType;
		oscillator.frequency.setValueAtTime(note.frequencyHz, startTime);

		gainNode.gain.setValueAtTime(0.0001, startTime);
		gainNode.gain.linearRampToValueAtTime(CUE_VOLUME, startTime + CUE_FADE_IN_S);
		gainNode.gain.setValueAtTime(CUE_VOLUME, fadeOutStartTime);
		gainNode.gain.exponentialRampToValueAtTime(0.0001, noteEndTime);

		oscillator.connect(gainNode);
		gainNode.connect(context.destination);
		oscillator.start(startTime);
		oscillator.stop(noteEndTime);

		oscillator.onended = () => {
			oscillator.disconnect();
			gainNode.disconnect();
		};

		startTime = noteEndTime + CUE_NOTE_GAP_S;
	}
}
