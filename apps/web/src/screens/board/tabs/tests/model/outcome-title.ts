/* @layer renderer-app @kind logic */
import type { TestOutcome } from '../../../../../data/posts';

const outcomeTitle = (outcome: TestOutcome, testId: string): string =>
  outcome === 'pass' ? `Run ${testId} and post it as pass` : `Say what you saw, then post ${testId} as ${outcome}`;

export { outcomeTitle };
