/* @layer renderer-app @kind logic */
import type { BadgeColor } from '@drizztdourden08/tessera';
import { FAULT_DOT, LEAD_DOT } from '../boards.constants';
import type { BoardLive, BoardRecord } from '../model';

const leadDot = (record: BoardRecord, live: BoardLive): { color: BadgeColor; tip: string } => {
  if (live.fault) return { color: FAULT_DOT, tip: `Lead: ${live.fault.reason}` };
  return { color: LEAD_DOT[record.lead], tip: `Lead: ${record.lead}` };
};

export { leadDot };
