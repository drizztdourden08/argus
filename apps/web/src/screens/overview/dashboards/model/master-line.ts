/* @layer renderer-app @kind logic */
import type { Card } from '../../../../data/types';
import type { RunRow } from '../../../../data/runs';
import { formatAgo } from '../../format';
import { LIVE_RUN_STATUSES } from './model.constants';
import type { MasterLine } from './model.type';

const runLine = (run: RunRow, now: number): MasterLine => {
  if (LIVE_RUN_STATUSES.includes(run.status)) {
    const since = formatAgo(run.startedAt ?? run.queuedAt, now);
    return { text: `${run.status} ${since}${run.lastLine ? `: ${run.lastLine}` : ''}`, tone: 'success', running: true };
  }
  const ended = formatAgo(run.endedAt, now);
  const exit = run.status === 'failed' ? ` (exit ${run.exitCode ?? '?'})` : '';
  return { text: `${run.status}${exit} ${ended} ago`, tone: run.status === 'failed' ? 'danger' : 'muted', running: false };
};

const masterLine = (card: Card, now: number): MasterLine => {
  const master = card.master;
  if (!master) {
    return card.masterMissing
      ? { text: 'no master launcher: the thread must set one', tone: 'danger', running: false }
      : { text: 'no master set', tone: 'muted', running: false };
  }
  return master.lastRun ? runLine(master.lastRun, now) : { text: '', tone: 'muted', running: false };
};

export { masterLine };
