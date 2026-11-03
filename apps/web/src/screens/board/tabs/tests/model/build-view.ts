/* @layer renderer-app @kind logic */
import { closedBundles } from './closed-bundles';
import { filtersActive } from './filters-active';
import { groupBundles } from './group-bundles';
import { groupByPlan } from './group-by-plan';
import { groupTests } from './group-tests';
import { isToRun } from './is-to-run';
import { matchesFilters } from './matches-filters';
import type { TestDoc, TestsView, ViewInput } from './model.type';
import { planCases } from './plan-cases';
import { planViews } from './plan-views';
import { resolveLaunch } from './resolve-launch';

const buildView = (input: ViewInput): TestsView => {
  const { builds, items, launchers, defs, handed, filters, prefs, conflicted } = input;
  const closedTags = new Set(builds.filter((build) => build.closedAt).map((build) => build.id));
  const all = input.tests;
  const open = all.filter((test) => !(test.build && closedTags.has(test.build)));
  const inPlan = planCases(items, filters.plan);
  const keep = (test: TestDoc): boolean => (inPlan === null || inPlan.has(test.id)) && matchesFilters(test, filters, conflicted.has(test.id));
  const narrowed = filtersActive(filters);
  const plans = planViews(items, filters.plan);
  const rows = open.filter(keep);
  const bundles = groupBundles({ tests: rows, builds, handed, narrowed }).map((group) => ({ group, launch: resolveLaunch(group.build, launchers, defs) }));
  const closed = closedBundles(all.filter((test) => test.build && closedTags.has(test.build)).filter(keep), builds)
    .filter((group) => group.tests.length > 0 || !narrowed);
  return {
    plans,
    byPlan: groupByPlan(prefs.group === 'plan' ? rows : [], plans),
    bundles,
    cards: prefs.group === 'item' || prefs.group === 'thread' ? groupTests(rows, prefs.group, items) : [],
    closed,
    toRun: open.filter(isToRun).length,
    total: open.length,
    filtered: narrowed,
    closedCount: closedTags.size,
  };
};

export { buildView };
