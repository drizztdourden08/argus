/* @layer renderer-app @kind logic */
import { AGENT_ANCHOR } from '../tabs/agents';
import { ANCHOR_PATTERN } from '../tabs/tests';

const anchorTab = (wanted: string): string | null => {
  if (ANCHOR_PATTERN.test(wanted)) return 'tests';
  return AGENT_ANCHOR.test(wanted) ? 'agents' : null;
};

export { anchorTab };
