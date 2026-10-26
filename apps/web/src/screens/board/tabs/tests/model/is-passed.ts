/* @layer renderer-app @kind logic */
import { PASSED_STATUSES } from './model.constants';
import type { TestDoc } from './model.type';

const isPassed = (test: TestDoc): boolean => PASSED_STATUSES.includes(test.status);

export { isPassed };
