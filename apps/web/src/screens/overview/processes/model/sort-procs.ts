/* @layer renderer-app @kind logic */
import { asNumber } from '../../../../format';
import { needsLook } from './needs-look';
import type { ProcItem } from './model.type';

const compareProcs = (a: ProcItem, b: ProcItem): number =>
  Number(needsLook(b)) - Number(needsLook(a))
  || (asNumber(b.memMB) ?? 0) - (asNumber(a.memMB) ?? 0)
  || String(a.name ?? '').localeCompare(String(b.name ?? ''))
  || (asNumber(a.pid) ?? 0) - (asNumber(b.pid) ?? 0);

const sortProcs = (procs: readonly ProcItem[]): ProcItem[] => [...procs].sort(compareProcs);

export { sortProcs };
