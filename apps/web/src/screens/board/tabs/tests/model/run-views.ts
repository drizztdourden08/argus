/* @layer renderer-app @kind logic */
import { compareIds } from './compare-ids';
import type { ItemDoc, LaunchDefDoc, LauncherDoc, RunView, TestDoc, TestFilters } from './model.type';
import { runClosed } from './run-closed';
import { runView } from './run-view';

interface RunsInput {
  items: readonly ItemDoc[];
  tests: readonly TestDoc[];
  launchers: readonly LauncherDoc[];
  defs: readonly LaunchDefDoc[];
  filters: TestFilters;
}

const rank = (view: RunView): number => {
  if (view.handed) return 0;
  return view.stage === 'draft' ? 2 : 1;
};

const byRank = (a: RunView, b: RunView): number => rank(a) - rank(b) || compareIds(b.item.id, a.item.id);

const matches = (view: RunView, filters: TestFilters): boolean => {
  if (filters.plan && view.plan?.id !== filters.plan) return false;
  const query = filters.query.trim().toLowerCase();
  if (!query) return true;
  return [view.item.id, view.plan?.id ?? '', view.plan?.title ?? '', view.build ?? ''].some((text) => text.toLowerCase().includes(query));
};

const runItems = (items: readonly ItemDoc[]): ItemDoc[] => items.filter((item) => item.type === 'test-run' && !item.deletedAt);

const runViews = (input: RunsInput): RunView[] =>
  runItems(input.items)
    .map((item) => runView({ ...input, item }))
    .filter((view) => matches(view, input.filters))
    .sort(byRank);

const closedOf = (views: readonly RunView[]): RunView[] => views.filter((view) => runClosed(view.stage));

const openOf = (views: readonly RunView[]): RunView[] => views.filter((view) => !runClosed(view.stage));

export { closedOf, openOf, runItems, runViews };
