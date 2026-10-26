/* @layer renderer-app @kind logic */
import { asNumber } from '../../format';
import type { ProcItem, ProcRow } from './model.type';

const toProcRow = (proc: ProcItem, nameOf: (key: string) => string): ProcRow => ({
  id: proc.id,
  rule: proc.rule ?? '-',
  name: proc.name ?? '?',
  pid: asNumber(proc.pid),
  project: proc.project ? nameOf(proc.project) : '',
  memMB: asNumber(proc.memMB),
  cpuPct: asNumber(proc.cpuPct),
  ageMin: asNumber(proc.ageMin),
  idleMin: asNumber(proc.idleMin),
  status: proc.status ?? '',
});

export { toProcRow };
