/* @layer renderer-app @kind logic */
import type { ProcDoc } from '../../../../data/types';
import { STATUS_TIP } from './model.constants';

const statusTip = (proc: ProcDoc): string => {
  const status = String(proc.status ?? '');
  const flags = Array.isArray(proc.flags) ? proc.flags.map(String) : [];
  const lines = [STATUS_TIP[status] ?? '', ...flags].filter(Boolean);
  return lines.length > 0 ? lines.join('\n') : status;
};

export { statusTip };
