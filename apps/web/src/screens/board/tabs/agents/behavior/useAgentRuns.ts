/* @layer renderer-app @kind logic */
import { useMemo } from 'react';
import { useCollection } from '../../../../../data/react';
import { newestFirst } from '../../../../../data/runs';
import type { RunDoc } from '../../../../../data/types';
import type { AgentRuns } from '../AgentsTab.type';

const useAgentRuns = (boardKey: string): AgentRuns => {
  const runs = useCollection<RunDoc>(boardKey, 'runs');
  return useMemo(() => {
    const sorted = [...runs.docs].sort(newestFirst);
    return { all: sorted };
  }, [runs.docs]);
};

export { useAgentRuns };
