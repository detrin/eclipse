// @ts-check
import { defineConfig } from 'astro/config';

import tailwindcss from '@tailwindcss/vite';

// https://astro.build/config
export default defineConfig({
  vite: {
    plugins: [tailwindcss()],
    optimizeDeps: {
      exclude: ['eclipse-wasm']
    },
    server: {
      fs: {
        // Allow serving files from the pkg directory
        allow: ['..']
      }
    }
  }
});