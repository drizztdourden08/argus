/* @layer renderer-app @kind logic */
import { isPassed } from './is-passed';
import { isToRun } from './is-to-run';
import type { TestDoc } from './model.type';
import { sortDone } from './sort-done';

const doneTests = (tests: readonly TestDoc[], showPassed: boolean): TestDoc[] =>
  sortDone(tests.filter((test) => !isToRun(test) && (showPassed || !isPassed(test))));

export { doneTests };
