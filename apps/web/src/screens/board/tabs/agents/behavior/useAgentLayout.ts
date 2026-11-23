/* @layer renderer-app @kind logic */
import { useStored } from '../../../shared';
import { LAYOUT_KEY } from '../model';
import type { AgentLayout } from '../model';

const useAgentLayout = (): [AgentLayout, (layout: AgentLayout) => void] => {
  const [stored, write] = useStored(LAYOUT_KEY);
  return [stored === 'cards' ? 'cards' : 'list', write];
};

export { useAgentLayout };
