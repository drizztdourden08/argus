/* @layer root-config @kind config */
import { defineBrockVitestConfig } from '@drizztdourden08/brock-build/vitest';

export default defineBrockVitestConfig(import.meta.dirname, {
  test: {
    environment: 'jsdom',
    include: ['tests/**/*.test.{ts,tsx}'],
    setupFiles: ['tests/helpers/setup-dom.ts'],
    // Every file imports the Tessera source barrel (about 5000 modules) into its own fork and
    // jsdom, so a fork holds several hundred MB and the CPU is the limit. Seven forks at once
    // starve each other and the first render of a file runs past the test timeout.
    maxWorkers: 3,
    // The first mount of a board tab compiles and renders a large Tessera tree; on a shared
    // machine that alone can pass the 5 s default.
    testTimeout: 15_000,
  },
});
