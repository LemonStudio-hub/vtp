import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';
import path from 'path';

export default defineConfig({
  plugins: [sveltekit()],
  resolve: {
    alias: {
      $proto: path.resolve('./proto')
    }
  },
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
