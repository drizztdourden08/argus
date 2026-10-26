/* @layer renderer-app @kind logic */
import { useCallback } from 'react';
import { useLocation, useNavigate } from 'react-router';
import { AGENT_ANCHOR, AGENTS_HASH } from '../AgentsTab.constants';
import type { AgentAnchor } from '../AgentsTab.type';

const useAgentAnchor = (): AgentAnchor => {
  const { hash, search } = useLocation();
  const navigate = useNavigate();
  const wanted = hash.slice(1);
  const open = useCallback((id: string) => void navigate({ search, hash: id }, { replace: true }), [navigate, search]);
  const close = useCallback(() => void navigate({ search, hash: AGENTS_HASH }, { replace: true }), [navigate, search]);
  return { selected: AGENT_ANCHOR.test(wanted) ? wanted : null, open, close };
};

export { useAgentAnchor };
