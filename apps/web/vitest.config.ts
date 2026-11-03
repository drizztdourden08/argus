/* @layer root-config @kind config */
import { resolve } from 'node:path';
import { defineBrockVitestConfig } from '@drizztdourden08/brock-build/vitest';
import { defineStandardsVitest } from '@drizztdourden08/standards/vitest';

// Brock's base keeps the automatic JSX runtime, the react dedupe and the inlined Tessera source;
// defineStandardsVitest adds the forks pool, the three tags with their timeouts and strictTags.
// Every file imports the Tessera source barrel (about 5000 modules) into its own fork and jsdom,
// so the CPU is the limit: the standard caps the workers at 3.
export default defineBrockVitestConfig(import.meta.dirname, {
  resolve: { alias: { '@argus/registry': resolve(import.meta.dirname, '../../crates/argus-board/registry') } },
  test: {
    environment: 'jsdom',
    include: ['tests/**/*.test.{ts,tsx}'],
    setupFiles: ['tests/helpers/setup-dom.ts'],
  },
}).then(defineStandardsVitest);
