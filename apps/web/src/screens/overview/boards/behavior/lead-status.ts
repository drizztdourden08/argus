/* @layer renderer-app @kind logic */
import type { AvatarStatus } from '@drizztdourden08/tessera';
import { LEAD_STATUS } from '../boards.constants';
import type { BoardLive, BoardRecord } from '../model';

const leadStatus = (record: BoardRecord, live: BoardLive): { status: AvatarStatus; tip: string } => {
  if (live.fault) return { status: 'fault', tip: `Lead: ${live.fault.reason}` };
  return { status: LEAD_STATUS[record.lead], tip: `Lead: ${record.lead}` };
};

export { leadStatus };
