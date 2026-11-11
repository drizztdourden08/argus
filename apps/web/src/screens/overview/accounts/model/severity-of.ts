/* @layer renderer-app @kind logic */
import type { AccountDoc, AccountSeverity } from '../../../../data/types';

const severityOf = (account: AccountDoc): AccountSeverity => {
  const levels = (account.limits ?? []).filter((limit) => limit.isActive !== false).map((limit) => limit.severity);
  if (levels.includes('critical')) return 'critical';
  return levels.includes('warning') ? 'warning' : 'normal';
};

export { severityOf };
