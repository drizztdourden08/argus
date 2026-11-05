/* @layer renderer-app @kind logic */
import { filtersActive } from './filters-active';
import { groupByPlan } from './group-by-plan';
import { groupTests } from './group-tests';
import { isToRun } from './is-to-run';
import { matchesFilters } from './matches-filters';
import type { TestDoc, TestsView, ViewInput } from './model.type';
import { planCases } from './plan-cases';
import { planViews } from './plan-views';
import { runCases } from './run-cases';
import { runClosed } from './run-closed';
import { tally } from './tally';
import { closedOf, openOf, runItems, runViews } from './run-views';

const testsView = (input: ViewInput): TestsView => {
  const { items, launchers, defs, filters, prefs, conflicted } = input;
  const inPlan = planCases(items, filters.plan);
  const inRuns = runCases(items, filters.runs);
  const keep = (test: TestDoc): boolean =>
    (inPlan === null || inPlan.has(test.id)) && (inRuns === null || inRuns.has(test.id)) && matchesFilters(test, filters, conflicted.has(test.id));
  const rows = input.tests.filter(keep);
  const plans = planViews(items, filters.plan);
  const runs = runViews({ items, tests: rows, launchers, defs, filters });
  const all = tally(input.tests);
  return {
    plans,
    byPlan: groupByPlan(prefs.group === 'plan' ? rows : [], plans),
    runs: openOf(runs),
    cards: prefs.group === 'item' || prefs.group === 'thread' ? groupTests(rows, prefs.group, items) : [],
    closed: closedOf(runs),
    toRun: input.tests.filter(isToRun).length,
    failed: all.fail,
    passed: all.pass,
    total: input.tests.length,
    filtered: filtersActive(filters),
    closedCount: runItems(items).filter((item) => runClosed(item.stage)).length,
  };
};

export { testsView };
