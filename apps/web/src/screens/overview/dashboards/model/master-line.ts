/* @layer renderer-app @kind logic */
import type { Card } from '../../../../data/types';
import type { RunRow } from '../../../../data/runs';
import { LIVE_RUN_STATUSES } from './model.constants';
import type { MasterLine } from './model.type';

const runLine = (run: RunRow): MasterLine => {
  if (LIVE_RUN_STATUSES.includes(run.status)) {
    return { head: run.status, at: run.startedAt ?? run.queuedAt, tail: run.lastLine ? `: ${run.lastLine}` : '', tone: 'success', running: true };
  }
  const exit = run.status === 'failed' ? ` (exit ${run.exitCode ?? '?'})` : '';
  return { head: `${run.status}${exit}`, at: run.endedAt, tail: '', tone: run.status === 'failed' ? 'danger' : 'muted', running: false };
};

const plain = (head: string, tone: MasterLine['tone']): MasterLine => ({ head, at: null, tail: '', tone, running: false });

const masterLine = (card: Card): MasterLine => {
  const master = card.master;
  if (!master) return card.masterMissing ? plain('no master launcher: the thread must set one', 'danger') : plain('no master set', 'muted');
  return master.lastRun ? runLine(master.lastRun) : plain('', 'muted');
};

export { masterLine };
