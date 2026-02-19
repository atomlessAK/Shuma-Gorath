import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';

const dashboardRoot = path.dirname(fileURLToPath(import.meta.url));

export default defineConfig({
  root: dashboardRoot,
  plugins: [sveltekit()]
});
