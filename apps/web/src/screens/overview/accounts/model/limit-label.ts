/* @layer renderer-app @kind logic */
import type { AccountLimit } from '../../../../data/types';
import { LIMIT_LABEL, SCOPED_KIND } from '../accounts.constants';

const limitLabel = (limit: AccountLimit): string => {
  if (limit.kind === SCOPED_KIND) return `Weekly, ${limit.model ?? 'one model'}`;
  return LIMIT_LABEL[limit.kind ?? ''] ?? limit.kind ?? 'Limit';
};

export { limitLabel };
