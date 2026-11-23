/* @layer renderer-app @kind logic */
import type { TestDoc } from './model.type';

const caseTime = (test: TestDoc): string => test.result?.at ?? test.updatedAt;

export { caseTime };
