/* @layer renderer-app @kind logic */
import { useMemo } from 'react';
import { useCollection } from '../../../../../data/react';
import { isLiveRun } from '../../../../../data/runs';
import type { RunDoc } from '../../../../../data/types';
import { FINISHED_RUNS_SHOWN } from '../AgentsTab.constants';
import type { AgentRuns } from '../AgentsTab.type';
import { newestFirst, runsByAgent } from '../model';

const useAgentRuns = (boardKey: string): AgentRuns => {
  const runs = useCollection<RunDoc>(boardKey, 'runs');
  return useMemo(() => {
    const sorted = [...runs.docs].sort(newestFirst);
    return {
      all: sorted,
      live: sorted.filter(isLiveRun),
      finished: sorted.filter((run) => !isLiveRun(run)).slice(0, FINISHED_RUNS_SHOWN),
      byAgent: runsByAgent(sorted),
    };
  }, [runs.docs]);
};

export { useAgentRuns };
