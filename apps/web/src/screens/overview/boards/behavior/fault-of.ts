/* @layer renderer-app @kind logic */
import type { RegistryFault, RegistryRow } from '../../../../data/types';

const faultOf = (rows: readonly RegistryRow[], key: string): RegistryFault | null => {
  const own = rows.filter((row) => row.project === key);
  const failed = own.find((row) => row.state === 'fault');
  if (failed) return failed.fault ?? { since: failed.since ?? '', reason: 'wake-failed' };
  const closed = own.find((row) => row.boardRequired === true && row.boardOpen !== true);
  return closed ? { since: closed.since ?? '', reason: 'board-closed' } : null;
};

export { faultOf };
