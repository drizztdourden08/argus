/* @layer renderer-app @kind logic */
import type { TestOutcome } from '../../../../../data/posts';
import type { TestDoc } from './model.type';

const noteOutcome = (test: TestDoc): TestOutcome | null => {
  if (test.status === 'pending') return null;
  return test.status === 'moved' ? 'pass' : test.status;
};

export { noteOutcome };
