/* @layer renderer-app @kind logic */
import type { ItemDoc } from './model.type';

const casesOf = (item: ItemDoc): string[] => {
  const cases = item.fields?.cases;
  return Array.isArray(cases) ? cases.filter((id): id is string => typeof id === 'string') : [];
};

const planCases = (items: readonly ItemDoc[], planId: string): ReadonlySet<string> | null => {
  if (!planId) return null;
  const plan = items.find((item) => item.id === planId && item.type === 'test-plan');
  return new Set(plan ? casesOf(plan) : []);
};

export { casesOf, planCases };
