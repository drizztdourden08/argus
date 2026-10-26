/* @layer renderer-app @kind logic */
import { FILTER_IDS, FILTER_KEY, RULES_KEY, RULES_STATES } from '../worktrees.constants';
import type { TabPrefs } from '../worktrees.type';
import { usePersisted } from './usePersisted';

const useTabPrefs = (): TabPrefs => {
  const [filter, setFilter] = usePersisted(FILTER_KEY, FILTER_IDS, 'all');
  const [rules, setRules] = usePersisted(RULES_KEY, RULES_STATES, '0');
  return { filter, setFilter, rulesOpen: rules === '1', setRulesOpen: (open) => setRules(open ? '1' : '0') };
};

export { useTabPrefs };
