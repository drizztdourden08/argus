/* @layer renderer-app @kind logic */
import { isToRun } from './is-to-run';
import type { BuildDoc, TestDoc } from './model.type';

const toRunCount = (tests: readonly TestDoc[], builds: readonly BuildDoc[]): number => {
  const closed = new Set(builds.filter((build) => build.closedAt).map((build) => build.id));
  return tests.filter((test) => !test.deletedAt && isToRun(test) && !(test.build && closed.has(test.build))).length;
};

export { toRunCount };
