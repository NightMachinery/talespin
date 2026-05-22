// See https://kit.svelte.dev/docs/types#app
// for information about these interfaces
// and what to do when importing types
declare module 'svelte-feather-icons/src/icons/*.svelte' {
	import type { SvelteComponent } from 'svelte';

	const component: typeof SvelteComponent;
	export default component;
}

declare namespace App {
	// interface Locals {}
	// interface PageData {}
	// interface Error {}
	// interface Platform {}
}
declare global {
	interface Window {
		gameServer: GameServer;
	}
}
