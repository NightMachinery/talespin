import { browser } from '$app/environment';
import {
	DEFAULT_CUE_VOLUME,
	getCueDefinition,
	type CueNote,
	type GameCueId,
	type StageCueStage
} from '$lib/stageCues';

type BrowserWindow = Window &
	typeof globalThis & {
		webkitAudioContext?: typeof AudioContext;
	};

const CUE_NOTE_GAP_S = 0.03;
const CUE_FADE_IN_S = 0.015;
const CUE_FADE_OUT_S = 0.04;

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
	await getRunningAudioContext();
}

export async function playCue(cueId: GameCueId) {
	const context = await getRunningAudioContext();
	if (!context) return;

	const cue = getCueDefinition(cueId);
	playCueNotes(
		context,
		cue.notes,
		cue.oscillatorType ?? 'sine',
		cue.volume ?? DEFAULT_CUE_VOLUME,
		context.currentTime + 0.01
	);
}

export async function playStageChangeCue(stage: StageCueStage) {
	await playCue(stage);
}

export async function playScoutTurnCue() {
	await playCue('ScoutTurn');
}

async function getRunningAudioContext() {
	const context = getAudioContext();
	if (!context) return null;

	if (context.state === 'suspended') {
		try {
			await context.resume();
		} catch {
			// Ignore autoplay-policy failures and keep gameplay uninterrupted.
			return null;
		}
	}

	return context.state === 'running' ? context : null;
}

function playCueNotes(
	context: AudioContext,
	notes: readonly CueNote[],
	oscillatorType: OscillatorType,
	volume: number,
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
		gainNode.gain.linearRampToValueAtTime(volume, startTime + CUE_FADE_IN_S);
		gainNode.gain.setValueAtTime(volume, fadeOutStartTime);
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
