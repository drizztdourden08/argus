/* @layer renderer-app @kind logic */
import type { RegistryRow } from '../../../../data/types';
import { STATE_RANK, UNKNOWN_RANK } from '../RegisteredStrip.constants';
import { isLeadRole } from '../../../../data/content';

const isLead = (row: RegistryRow): number => (isLeadRole(row.role) ? -1 : 0);

const nameOf = (row: RegistryRow): string => row.session ?? row.thread;

const compareRows = (a: RegistryRow, b: RegistryRow): number =>
  (STATE_RANK[a.state] ?? UNKNOWN_RANK) - (STATE_RANK[b.state] ?? UNKNOWN_RANK) ||
  isLead(a) - isLead(b) ||
  nameOf(a).localeCompare(nameOf(b));

const sortRows = (rows: RegistryRow[]): RegistryRow[] => [...rows].sort(compareRows);

export { sortRows };
