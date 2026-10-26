/* @layer renderer-app @kind logic */
import { isToRun } from './is-to-run';
import type { Counts, TestDoc } from './model.type';

const settled = (test: TestDoc): keyof Counts | null => {
  const status = test.status === 'moved' ? 'pass' : test.status;
  return status === 'pass' || status === 'fail' || status === 'partial' || status === 'blocked' ? status : null;
};

const tally = (tests: readonly TestDoc[]): Counts => {
  const counts: Counts = { run: 0, pass: 0, fail: 0, partial: 0, blocked: 0 };
  for (const test of tests) {
    const key = isToRun(test) ? 'run' : settled(test);
    if (key) counts[key] += 1;
  }
  return counts;
};

export { tally };
