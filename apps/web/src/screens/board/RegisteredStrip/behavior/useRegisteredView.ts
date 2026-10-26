/* @layer renderer-app @kind logic */
import { useMemo } from 'react';
import { useCollection } from '../../../../data/react';
import type { RegistryRow } from '../../../../data/types';
import type { RegisteredView, RegistryAgent } from '../RegisteredStrip.type';
import { agentLoads } from './agent-loads';
import { sortRows } from './sort-rows';

const useRegisteredView = (key: string): RegisteredView => {
  const registry = useCollection<RegistryRow>(key, 'registry');
  const agents = useCollection<RegistryAgent>(key, 'agents');
  return useMemo(
    () => ({ rows: sortRows(registry.docs), loads: agentLoads(agents.docs, Date.now()) }),
    [registry.docs, agents.docs],
  );
};

export { useRegisteredView };
