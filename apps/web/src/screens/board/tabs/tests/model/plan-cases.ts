/* @layer renderer-app @kind logic */
import type { ItemDoc } from './model.type';

const planCases = (items: readonly ItemDoc[], planId: string): ReadonlySet<string> | null => {
  if (!planId) return null;
  const cases = items.find((item) => item.id === planId && item.type === 'test-plan')?.fields?.cases;
  return new Set(Array.isArray(cases) ? cases.filter((id): id is string => typeof id === 'string') : []);
};

export { planCases };
