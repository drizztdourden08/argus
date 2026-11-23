/* @layer renderer-app @kind logic */
import type { ChatTranscriptMessage } from '@drizztdourden08/tessera';
import { createElement } from 'react';
import { TALK_ROLE, TALK_WHO } from '../OverlordChat.constants';
import type { TalkRow, ToolsRow, TranscriptRow } from '../OverlordChat.type';
import { StepContent } from '../sub-components/StepContent';
import { ToolsGroup } from '../sub-components/ToolsGroup';
import { clockText } from '../../../screens/overview/accounts/model/clock-text';
import { STEP_KINDS } from './to-messages.constants';

const timeOf = (row: TranscriptRow): { time?: string } => {
  const time = clockText(row.at);
  return time ? { time } : {};
};

const toolsMessage = (row: ToolsRow): ChatTranscriptMessage => ({
  id: row.key,
  speaker: TALK_WHO.assistant,
  role: TALK_ROLE.assistant,
  content: createElement(ToolsGroup, { tools: row.tools }),
  ...timeOf(row),
});

const talkMessage = (row: TalkRow): ChatTranscriptMessage => ({
  id: row.key,
  speaker: TALK_WHO[row.kind],
  role: TALK_ROLE[row.kind],
  content: STEP_KINDS.includes(row.kind) ? createElement(StepContent, { row }) : row.text,
  ...timeOf(row),
});

const toMessage = (row: TranscriptRow): ChatTranscriptMessage => (row.kind === 'tools' ? toolsMessage(row) : talkMessage(row));

const toMessages = (rows: readonly TranscriptRow[]): ChatTranscriptMessage[] => rows.map(toMessage);

export { toMessages };
