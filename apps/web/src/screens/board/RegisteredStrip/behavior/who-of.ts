/* @layer renderer-app @kind logic */
import type { RegistryRow } from '../../../../data/types';
import { isLeadRole } from '../../../../data/content';

const whoOf = (row: RegistryRow): string => {
  const name = row.session ?? row.thread;
  return isLeadRole(row.role) ? `${name} (lead)` : name;
};

export { whoOf };
