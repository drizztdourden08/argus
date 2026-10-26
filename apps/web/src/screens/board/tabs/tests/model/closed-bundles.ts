/* @layer renderer-app @kind logic */
import type { BuildDoc, ClosedGroup, TestDoc } from './model.type';
import { tally } from './tally';

const closedBundles = (tests: readonly TestDoc[], builds: readonly BuildDoc[]): ClosedGroup[] =>
  builds
    .filter((build) => build.closedAt)
    .sort((a, b) => String(b.closedAt).localeCompare(String(a.closedAt)))
    .map((build) => {
      const rows = tests.filter((test) => test.build === build.id);
      return { tag: build.id, build, tests: rows, counts: tally(rows) };
    });

export { closedBundles };
