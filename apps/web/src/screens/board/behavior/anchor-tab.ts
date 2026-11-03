/* @layer renderer-app @kind logic */
import { RETIRED_TABS } from '../board.constants';
import { AGENT_ANCHOR } from '../tabs/agents/AgentsTab.constants';
import { ANCHOR_PATTERN } from '../tabs/tests/tests.constants';

const anchorTab = (wanted: string): string | null => {
  if (ANCHOR_PATTERN.test(wanted)) return 'tests';
  if (AGENT_ANCHOR.test(wanted)) return 'agents';
  return RETIRED_TABS.get(wanted) ?? null;
};

export { anchorTab };
