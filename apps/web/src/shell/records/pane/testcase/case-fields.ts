/* @layer renderer-app @kind logic */
import type { Test } from '../../../../data/types';

const caseFields = (test: Test): Readonly<Record<string, unknown>> => ({
  setup: test.setup.map((entry) => `${entry.label}: ${entry.text}`),
  steps: test.steps,
  pass: test.pass,
  fail: test.fail,
  mode: test.mode,
  bundle: test.build ?? '',
});

export { caseFields };
