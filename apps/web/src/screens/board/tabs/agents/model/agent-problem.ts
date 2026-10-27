/* @layer renderer-app @kind logic */
import { AGENT_LIMITS } from './model.constants';
import type { AgentField } from './model.type';

const agentProblem = (field: AgentField, text: string): string | null => {
  const trimmed = text.trim();
  if (field === 'task' && trimmed === '') return 'A task cannot be empty.';
  const limit = AGENT_LIMITS[field];
  return trimmed.length > limit ? `At most ${limit} characters.` : null;
};

export { agentProblem };
