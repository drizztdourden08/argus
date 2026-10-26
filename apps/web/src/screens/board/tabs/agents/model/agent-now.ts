/* @layer renderer-app @kind logic */
import type { Agent, AgentTool } from '../../../../../data/agents';
import { isActive } from './is-active';
import { FILE_TOOLS } from './model.constants';
import type { NowLine } from './model.type';

const baseName = (path: string): string => path.split(/[\\/]/).filter(Boolean).pop() ?? '';

const toolText = (tool: AgentTool | null | undefined): string => {
  if (!tool?.name) return 'tool';
  const summary = FILE_TOOLS.includes(tool.name) ? baseName(tool.summary ?? '') : (tool.summary ?? '');
  return summary ? `${tool.name} ${summary}` : tool.name;
};

const agentNow = (agent: Agent): NowLine | null => {
  const live = agent.live;
  if (!live || !isActive(agent)) return null;
  if (live.activity === 'tool') return { text: toolText(live.lastTool), since: live.activitySince ?? null };
  if (live.activity === 'thinking') return { text: 'thinking', since: live.activitySince ?? null };
  if (live.activity === 'waiting') return { text: 'waiting', since: null };
  return live.liveNote ? { text: live.liveNote, since: null } : null;
};

export { agentNow };
