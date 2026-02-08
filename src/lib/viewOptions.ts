import { writable } from 'svelte/store';

// Default to scrollable cards for consistent gameplay presentation.
export const cardsFitToHeight = writable(false);
