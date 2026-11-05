/* @layer renderer-app @kind logic */
import { OWNER_ASSIGNEE } from './model.constants';
import type { ItemDoc, LaunchDefDoc, LauncherDoc, RunView, TestDoc } from './model.type';
import { casesOf } from './plan-cases';
import { resolveLaunch } from './resolve-launch';
import { runClosed } from './run-closed';
import { runCounts } from './run-counts';

interface RunViewInput {
  item: ItemDoc;
  items: readonly ItemDoc[];
  tests: readonly TestDoc[];
  launchers: readonly LauncherDoc[];
  defs: readonly LaunchDefDoc[];
}

const textOf = (value: unknown): string => (typeof value === 'string' ? value : '');

const runView = (input: RunViewInput): RunView => {
  const { item, items, tests, launchers, defs } = input;
  const plan = items.find((candidate) => candidate.id === textOf(item.fields?.plan) && candidate.type === 'test-plan') ?? null;
  const cases = plan ? casesOf(plan) : [];
  const build = textOf(item.fields?.build) || null;
  const stage = item.stage ?? 'draft';
  const handed = item.assignee === OWNER_ASSIGNEE && !runClosed(stage);
  return {
    item,
    plan,
    build,
    stage,
    handed,
    tests: tests.filter((test) => cases.includes(test.id)),
    counts: runCounts(item.fields?.results, cases),
    launch: resolveLaunch(build, launchers, defs),
    open: handed || stage === 'running',
  };
};

export { runView };
