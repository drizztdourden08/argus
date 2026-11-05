/* @layer renderer-app @kind logic */
import type { ItemDoc } from './model.type';
import { casesOf } from './plan-cases';

const textOf = (value: unknown): string => (typeof value === 'string' ? value : '');

const casesOfRun = (items: readonly ItemDoc[], run: ItemDoc): string[] => {
  const plan = items.find((item) => item.id === textOf(run.fields?.plan) && item.type === 'test-plan');
  return plan ? casesOf(plan) : [];
};

const runCases = (items: readonly ItemDoc[], runIds: readonly string[]): ReadonlySet<string> | null => {
  if (runIds.length === 0) return null;
  const runs = items.filter((item) => item.type === 'test-run' && runIds.includes(item.id));
  return new Set(runs.flatMap((run) => casesOfRun(items, run)));
};

export { runCases };
