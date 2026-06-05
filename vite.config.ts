import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';

export default defineConfig({
  plugins: [sveltekit()],
  server: {
    port: 5173,
    strictPort: false,
    fs: {
      allow: ['.']
    }
  },
  build: {
    target: 'esnext',
    minify: 'esbuild',
    sourcemap: true
  },
  worker: {
    format: 'es'
  },
  optimizeDeps: {
    exclude: ['vtp-core']
  },
  ssr: {
    noExternal: []
  }
});
