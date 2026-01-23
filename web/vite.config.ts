import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig, type UserConfig } from 'vite';
import wasm from 'vite-plugin-wasm';
import topLevelAwait from 'vite-plugin-top-level-await';

export default defineConfig({
  plugins: [
    sveltekit(),
    wasm(),
    topLevelAwait()
  ],
  server: {
    port: 28082,
    strictPort: false,  // Auto-increment if port is busy
    proxy: {
      '/api': {
        target: 'http://localhost:7000',
        changeOrigin: true,
        secure: false,
      }
    },
    // Enable headers for WASM SharedArrayBuffer
    headers: {
      'Cross-Origin-Opener-Policy': 'same-origin',
      'Cross-Origin-Embedder-Policy': 'require-corp'
    }
  },
  optimizeDeps: {
    exclude: ['@sveltejs/kit']
  }
} satisfies UserConfig);
