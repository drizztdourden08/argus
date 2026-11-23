/* @layer renderer-app @kind logic */
import type { RunRow } from '../../../../data/runs';
import { formatAgo } from '../../../../format';
import type { MasterLine } from '../model';

const masterWords = (run: RunRow, line: MasterLine, now: number): string => {
  if (run.status === 'queued') return 'Queued';
  if (run.status === 'running') return `Running for ${formatAgo(line.at, now)}`;
  return `Last run ${run.status} ${formatAgo(line.at, now)} ago`;
};

export { masterWords };
