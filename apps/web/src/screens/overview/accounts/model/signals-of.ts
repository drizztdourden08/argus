/* @layer renderer-app @kind logic */
import type { SignalItem } from '@drizztdourden08/tessera';
import type { AccountDoc } from '../../../../data/types';
import { DRIFT_TIP } from '../accounts.constants';
import { severityOf } from './severity-of';

const nameOf = (account: AccountDoc): string => account.label ?? account.key ?? 'Account';

const signalsOf = (accounts: readonly AccountDoc[], drift: number): SignalItem[] => {
  const items: SignalItem[] = accounts.flatMap((account): SignalItem[] => {
    const severity = severityOf(account);
    const id = account.key ?? nameOf(account);
    if (severity === 'critical') return [{ id, icon: 'triangle-alert', label: `${nameOf(account)} is at a limit`, tone: 'danger' }];
    if (severity === 'normal') return [{ id, icon: 'circle-check', label: `${nameOf(account)} has room`, tone: 'success' }];
    return [];
  });
  items.push({ id: 'drift', icon: 'split', count: drift, label: DRIFT_TIP, tone: drift > 0 ? 'warning' : 'neutral' });
  return items;
};

export { signalsOf };
