/* @layer renderer-app @kind component */
import { Disclosure, Text } from '@drizztdourden08/tessera';
import type { ChatTranscriptMessage } from '@drizztdourden08/tessera';
import { TALK_ROLE, TALK_WHO, THINKING_LABEL } from '../OverlordChat.constants';
import type { TalkRow, TranscriptRow } from '../OverlordChat.type';

const muted = (text: string) => <Text variant="caption" tone="muted">{text}</Text>;

const contentOf = (row: TalkRow) => {
  if (row.kind === 'thinking') return <Disclosure size="sm" summary={muted(THINKING_LABEL)}>{muted(row.text)}</Disclosure>;
  return row.kind === 'turn' || row.kind === 'approval' ? muted(row.text) : row.text;
};

const toMessage = (row: TranscriptRow): ChatTranscriptMessage => {
  if (row.kind === 'tools') {
    const tools = row.tools.map((tool) => ({ id: tool.id, name: tool.name, summary: tool.summary || undefined, status: tool.status }));
    return { id: row.key, speaker: TALK_WHO.assistant, role: TALK_ROLE.assistant, tools };
  }
  return { id: row.key, speaker: TALK_WHO[row.kind], role: TALK_ROLE[row.kind], content: contentOf(row) };
};

const toMessages = (rows: readonly TranscriptRow[]): ChatTranscriptMessage[] => rows.map(toMessage);

export { toMessages };
