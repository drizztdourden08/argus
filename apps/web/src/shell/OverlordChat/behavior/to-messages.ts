/* @layer renderer-app @kind logic */
import type { ChatTranscriptMessage } from '@drizztdourden08/tessera';
import { createElement } from 'react';
import { TALK_ROLE, TALK_WHO } from '../OverlordChat.constants';
import type { TalkRow, ToolsRow, TranscriptRow } from '../OverlordChat.type';
import { StepContent } from '../sub-components/StepContent';
import { ToolsGroup } from '../sub-components/ToolsGroup';
import { STEP_KINDS } from './to-messages.constants';

const toolsMessage = (row: ToolsRow): ChatTranscriptMessage => ({
  id: row.key,
  speaker: TALK_WHO.assistant,
  role: TALK_ROLE.assistant,
  content: createElement(ToolsGroup, { tools: row.tools }),
});

const talkMessage = (row: TalkRow): ChatTranscriptMessage => ({
  id: row.key,
  speaker: TALK_WHO[row.kind],
  role: TALK_ROLE[row.kind],
  content: STEP_KINDS.includes(row.kind) ? createElement(StepContent, { row }) : row.text,
});

const toMessage = (row: TranscriptRow): ChatTranscriptMessage => (row.kind === 'tools' ? toolsMessage(row) : talkMessage(row));

const toMessages = (rows: readonly TranscriptRow[]): ChatTranscriptMessage[] => rows.map(toMessage);

export { toMessages };
