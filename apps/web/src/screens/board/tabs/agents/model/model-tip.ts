/* @layer renderer-app @kind logic */
import type { Agent } from '../../../../../data/agents';
import { NO_WHY } from './model.constants';
import { orText } from './or-text';

const modelTip = (agent: Agent): string => {
  const running = agent.live?.modelUsed ? ` (running on ${agent.live.modelUsed})` : '';
  return `${orText(agent.modelWhy, NO_WHY)}${running}`;
};

export { modelTip };
