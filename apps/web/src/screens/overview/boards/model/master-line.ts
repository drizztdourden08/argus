/* @layer renderer-app @kind logic */
import { isLiveRun, runLine } from '../../../../data/runs';
import type { RunRow } from '../../../../data/runs';
import type { Card } from '../../../../data/types';
import type { MasterLine } from './board.type';

const lineOf = (run: RunRow): MasterLine => {
  const line = runLine(run);
  return { head: line.label, at: line.since, tail: line.last ? ` · ${line.last}` : '', tone: line.tone, running: isLiveRun(run) };
};

const plain = (head: string, tone: MasterLine['tone']): MasterLine => ({ head, at: null, tail: '', tone, running: false });

const masterLine = (card: Card): MasterLine => {
  const master = card.master;
  if (!master) return card.masterMissing ? plain('no master launcher: the thread must set one', 'danger') : plain('no master set', 'neutral');
  return master.lastRun ? lineOf(master.lastRun) : plain('', 'neutral');
};

export { masterLine };
