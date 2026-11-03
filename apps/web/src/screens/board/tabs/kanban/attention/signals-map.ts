/* @layer renderer-app @kind logic */
import { itemTests } from '../model/item-tests';
import { FAILING_STATUSES } from './attention.constants';
import type { Signals, SignalsInput } from './attention.type';

const bump = (map: Map<string, Signals>, item: string | null | undefined, field: keyof Signals): void => {
  if (!item) return;
  const signals = map.get(item) ?? { failing: 0, pending: 0 };
  signals[field] += 1;
  map.set(item, signals);
};

const signalsMap = (input: SignalsInput): Map<string, Signals> => {
  const map = new Map<string, Signals>();
  for (const test of itemTests(input.tests, input.closedBuilds, false)) {
    if (FAILING_STATUSES.includes(test.status)) bump(map, test.item, 'failing');
    else if (test.status === 'pending') bump(map, test.item, 'pending');
  }
  return map;
};

export { signalsMap };
