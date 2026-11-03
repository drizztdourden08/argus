/* @layer renderer-app @kind logic */
import { AGENT_ANCHOR } from '../tabs/agents/AgentsTab.constants';
import { ANCHOR_PATTERN } from '../tabs/tests/tests.constants';

const anchorTab = (wanted: string): string | null => {
  if (ANCHOR_PATTERN.test(wanted)) return 'tests';
  return AGENT_ANCHOR.test(wanted) ? 'agents' : null;
};

export { anchorTab };
