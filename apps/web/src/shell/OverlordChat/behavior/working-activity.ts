/* @layer renderer-app @kind logic */
import { RUNNING_WORD, THINKING_LABEL, WRITING_LABEL } from '../OverlordChat.constants';
import type { LiveDraft, ToolItem, TranscriptRow, WorkingActivity } from '../OverlordChat.type';

const runningTool = (rows: readonly TranscriptRow[]): ToolItem | undefined => {
  const last = rows.at(-1);
  return last?.kind === 'tools' ? last.tools.filter((tool) => tool.status === 'running').at(-1) : undefined;
};

const toolText = (name: string, summary: string): string => `${RUNNING_WORD} ${name}${summary ? `: ${summary}` : ''}`;

const workingActivity = (rows: readonly TranscriptRow[], draft: LiveDraft): WorkingActivity => {
  const tool = runningTool(rows);
  if (tool) return { text: toolText(tool.name, tool.summary), mark: 'working' };
  if (draft.tool) return { text: toolText(draft.tool, ''), mark: 'working' };
  const writing = draft.rows.at(-1)?.kind === 'assistant';
  return { text: writing ? WRITING_LABEL : THINKING_LABEL, mark: 'thinking' };
};

export { workingActivity };
