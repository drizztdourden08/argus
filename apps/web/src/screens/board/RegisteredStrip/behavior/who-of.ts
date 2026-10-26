/* @layer renderer-app @kind logic */
import type { RegistryRow } from '../../../../data/types';

const whoOf = (row: RegistryRow): string => {
  const name = row.session ?? row.thread;
  return row.role === 'coordinator' ? `${name} (lead)` : name;
};

export { whoOf };
