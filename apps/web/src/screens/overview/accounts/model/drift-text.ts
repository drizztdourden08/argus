/* @layer renderer-app @kind logic */
import type { AccountsDrift } from '../../../../data/types';
import { BOTH_MATCH, DRIFT_KINDS } from '../compare.constants';
import { plural } from './plural';

const driftText = (drift: AccountsDrift | null): string => {
  const total = drift?.total ?? 0;
  if (total === 0) return BOTH_MATCH;
  const parts = Object.entries(DRIFT_KINDS).flatMap(([kind, nouns]) => {
    const count = drift?.byKind?.[kind] ?? 0;
    return count > 0 ? [plural(count, nouns[0], nouns[1])] : [];
  });
  return `${plural(total, 'difference', 'differences')}${parts.length > 0 ? `: ${parts.join(', ')}` : ''}`;
};

export { driftText };
