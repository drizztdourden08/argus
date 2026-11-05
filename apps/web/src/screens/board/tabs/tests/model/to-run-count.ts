/* @layer renderer-app @kind logic */
import { isToRun } from './is-to-run';
import type { TestDoc } from './model.type';

const toRunCount = (tests: readonly TestDoc[]): number => tests.filter((test) => !test.deletedAt && isToRun(test)).length;

export { toRunCount };
