/* @layer renderer-app @kind logic */
import type { CardTone } from '@drizztdourden08/tessera';
import type { Counts } from './model.type';

const planTone = (counts: Counts): CardTone | undefined => {
  if (counts.fail > 0) return 'danger';
  if (counts.partial > 0 || counts.blocked > 0) return 'warning';
  const settled = counts.pass > 0 && counts.run === 0;
  return settled ? 'success' : undefined;
};

export { planTone };
