/* @layer renderer-app @kind logic */
import type { ChatTranscriptMessage } from '@drizztdourden08/tessera';
import { TALK_ROLE, TALK_WHO } from '../OverlordChat.constants';
import type { TranscriptRow } from '../OverlordChat.type';

const toMessage = (row: TranscriptRow): ChatTranscriptMessage => {
  if (row.kind === 'tools') {
    return { id: row.key, speaker: TALK_WHO.assistant, role: TALK_ROLE.assistant, tools: row.names.map((name, at) => ({ id: `${row.key}-${at}`, name })) };
  }
  return { id: row.key, speaker: TALK_WHO[row.kind], role: TALK_ROLE[row.kind], content: row.text };
};

const toMessages = (rows: readonly TranscriptRow[]): ChatTranscriptMessage[] => rows.map(toMessage);

export { toMessages };
