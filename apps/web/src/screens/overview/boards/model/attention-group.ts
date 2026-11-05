/* @layer renderer-app @kind logic */
import type { AttentionGroupId, BoardLive, BoardRecord } from './board.type';

const countsNeedYou = ({ counts }: BoardRecord): boolean => counts.needsYou > 0 || counts.testsFailing > 0 || counts.blocked > 0;

const liveNeedsYou = (live?: BoardLive): boolean => Boolean(live?.fault) || (live?.stalls ?? 0) > 0;

const attentionGroup = (record: BoardRecord, live?: BoardLive): AttentionGroupId => {
  if (record.archived) return 'archived';
  if (!record.started) return 'not-started';
  if (countsNeedYou(record) || liveNeedsYou(live)) return 'needs-you';
  return record.masterRunning || record.lead === 'running' ? 'running' : 'quiet';
};

export { attentionGroup };
