import { purgeCss } from 'vite-plugin-tailwind-purgecss';
import { sveltekit } from '@sveltejs/kit/vite';
import { configDefaults, defineConfig } from 'vitest/config';

export default defineConfig({
	plugins: [sveltekit(), purgeCss()],
	envPrefix: ['VITE_', 'TALESPIN_'],
	build: {
		chunkSizeWarningLimit: 700
	},
	test: {
		exclude: [...configDefaults.exclude, '**/.worktrees/**']
	}
});
