/* @layer renderer-app @kind logic */
import type { AccountLimit } from '../../../../data/types';
import { KIND_LABEL } from '../accounts.constants';

const limitLabel = (limit: AccountLimit): string => {
  const kind = limit.kind ?? 'limit';
  const base = KIND_LABEL[kind] ?? kind.replaceAll('_', ' ');
  return limit.model ? `${base} (${limit.model})` : base;
};

export { limitLabel };
