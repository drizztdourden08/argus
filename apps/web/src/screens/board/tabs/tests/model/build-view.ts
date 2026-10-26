/* @layer renderer-app @kind logic */
import { closedBundles } from './closed-bundles';
import { filtersActive } from './filters-active';
import { groupBundles } from './group-bundles';
import { groupTests } from './group-tests';
import { isToRun } from './is-to-run';
import { matchesFilters } from './matches-filters';
import type { TestDoc, TestsView, ViewInput } from './model.type';
import { resolveLaunch } from './resolve-launch';

const buildView = (input: ViewInput): TestsView => {
  const { builds, items, launchers, defs, handed, filters, prefs, conflicted } = input;
  const closedTags = new Set(builds.filter((build) => build.closedAt).map((build) => build.id));
  const all = input.tests;
  const open = all.filter((test) => !(test.build && closedTags.has(test.build)));
  const keep = (test: TestDoc): boolean => matchesFilters(test, filters, conflicted.has(test.id));
  const narrowed = filtersActive(filters);
  const rows = open.filter(keep);
  const bundles = groupBundles({ tests: rows, builds, handed, narrowed }).map((group) => ({ group, launch: resolveLaunch(group.build, launchers, defs) }));
  const closed = closedBundles(all.filter((test) => test.build && closedTags.has(test.build)).filter(keep), builds)
    .filter((group) => group.tests.length > 0 || !narrowed);
  return {
    bundles,
    cards: prefs.group === 'build' ? [] : groupTests(rows, prefs.group, items),
    closed,
    toRun: open.filter(isToRun).length,
    total: open.length,
    filtered: narrowed,
    closedCount: closedTags.size,
  };
};

export { buildView };
