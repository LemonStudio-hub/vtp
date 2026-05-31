import { defineConfig } from 'vitest/config';
import { sveltekit } from '@sveltejs/kit/vite';

export default defineConfig({
  plugins: [sveltekit()],
  test: {
    include: ['src/**/*.test.ts', 'tests/**/*.test.ts'],
    environment: 'jsdom',
    globals: true,
    setupFiles: ['./tests/setup.ts'],
    coverage: {
      provider: 'v8',
      reporter: ['text', 'json', 'html'],
      include: ['src/**/*.{ts,svelte}'],
      exclude: ['src/lib/vtp-core/**', 'src/**/*.d.ts']
    }
  },
  resolve: {
    alias: {
      $stores: '/home/dev/projects/vtp/src/stores',
      $utils: '/home/dev/projects/vtp/src/utils'
    }
  }
});
