/* @layer renderer-app @kind logic */
import type { AttentionGroupId, BoardLive, BoardRecord } from './board.type';

const attentionGroup = (record: BoardRecord, live?: BoardLive): AttentionGroupId => {
  if (record.archived) return 'archived';
  if (!record.started) return 'not-started';
  const { counts } = record;
  if (counts.needsYou > 0 || counts.testsFailing > 0 || counts.blocked > 0 || Boolean(live?.fault) || (live?.stalls ?? 0) > 0) return 'needs-you';
  return record.masterRunning || record.lead === 'running' ? 'running' : 'quiet';
};

export { attentionGroup };
