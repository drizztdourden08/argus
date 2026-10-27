/* @layer renderer-app @kind logic */
import { useCallback, useMemo } from 'react';
import { toast } from '@drizztdourden08/tessera';
import { bulkAgents, cancelStop, requestStop } from '../../../../../data/agents';
import type { AgentActions, AgentsView } from '../AgentsTab.type';
import { agentName, postVerdict } from '../model';
import { announce } from './announce';
import { announceFailure } from './announce-failure';
import { useAgentEdits } from './useAgentEdits';
import { useLatestAgents } from './useLatestAgents';

const useAgentActions = (boardKey: string, view: AgentsView): AgentActions => {
  const latest = useLatestAgents(view);
  const edits = useAgentEdits(boardKey, latest);

  const cancel = useCallback((id: string, undoes?: string) => {
    const agent = latest.current.get(id);
    if (!agent) return;
    cancelStop(boardKey, id, agent, undoes).then(
      (result) => announce(postVerdict(result, `Stop request cancelled for ${agentName(agent, id)}.`), `cancel-${id}`),
      announceFailure,
    );
  }, [boardKey, latest]);

  const stop = useCallback((id: string) => {
    const agent = latest.current.get(id);
    if (!agent) return;
    requestStop(boardKey, id, agent).then(({ entryId, result }) => {
      const done = `Stop requested for ${agentName(agent, id)}. The lead is told at its next check.`;
      announce(postVerdict(result, done), `stop-${entryId}`, () => cancel(id, entryId));
    }, announceFailure);
  }, [boardKey, cancel, latest]);

  const archiveFinished = useCallback((ids: readonly string[]) => {
    if (ids.length === 0) {
      toast({ message: 'No finished agent to archive.' });
      return;
    }
    bulkAgents(boardKey, 'archive', ids).then(({ entryIds, results }) => {
      const failed = results.map((result) => postVerdict(result, '')).find((verdict) => !verdict.ok);
      const undo = () => void bulkAgents(boardKey, 'unarchive', ids, entryIds).catch(announceFailure);
      announce(failed ?? { ok: true, message: `${ids.length} finished ${ids.length === 1 ? 'agent' : 'agents'} archived.` }, `archive-${entryIds.join('-')}`, undo);
    }, announceFailure);
  }, [boardKey]);

  return useMemo(() => ({ ...edits, stop, cancelStop: (id: string) => cancel(id), archiveFinished }), [edits, stop, cancel, archiveFinished]);
};

export { useAgentActions };
