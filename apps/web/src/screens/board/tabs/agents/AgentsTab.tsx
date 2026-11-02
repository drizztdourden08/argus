/* @layer renderer-app @kind component */
import { Box, Flex } from '@drizztdourden08/tessera';
import { useState } from 'react';
import { RunLogDrawer } from '../../run-terminal';
import { SECOND_TICK_MS, useNow, useStoredFlag } from '../../shared';
import { FINISHED_KEY } from './AgentsTab.constants';
import type { AgentsTabProps } from './AgentsTab.type';
import { useAgentActions } from './behavior/useAgentActions';
import { useAgentAnchor } from './behavior/useAgentAnchor';
import { useAgentRuns } from './behavior/useAgentRuns';
import { useAgentsView } from './behavior/useAgentsView';
import { useReconcile } from './behavior/useReconcile';
import { useRowState } from './behavior/useRowState';
import { AgentDrawer } from './sub-components/AgentDrawer';
import { AgentRunsSection } from './sub-components/AgentRunsSection';
import { AgentsBody } from './sub-components/AgentsBody';
import { AgentsFoot } from './sub-components/AgentsFoot';
import { AgentsHeader } from './sub-components/AgentsHeader';
import { ChatsList } from './sub-components/ChatsList';
import { ReconcilePanel } from './sub-components/ReconcilePanel';
import './AgentsTab.css';

const AgentsTab = (props: AgentsTabProps) => {
  const { boardKey, onPurge } = props;
  const view = useAgentsView(boardKey);
  const actions = useAgentActions(boardKey, view);
  const findings = useReconcile(boardKey, view.flagSignature);
  const now = useNow(view.active.length > 0);
  const runs = useAgentRuns(boardKey);
  const runNow = useNow(runs.live.length > 0, SECOND_TICK_MS);
  const state = useRowState();
  const anchor = useAgentAnchor();
  const [showFinished, setShowFinished] = useStoredFlag(FINISHED_KEY);
  const [logId, setLogId] = useState<string | null>(null);
  const opened = [...view.active, ...view.done].find((agent) => agent.id === anchor.selected) ?? null;
  const count = view.finishedCount;
  const order = [...view.active, ...(showFinished ? view.done : [])].map((agent) => agent.id);
  return (
    <Flex direction="column" gap="md">
      <ReconcilePanel findings={findings} />
      <Box as="section" aria-label="Agents" className="agents-panel">
        <AgentsHeader active={view.active} lead={view.lead} now={now} />
        <AgentRunsSection title="Running now" runs={runs.live} boardKey={boardKey} now={runNow} onLog={setLogId} />
        <AgentsBody
          active={view.active}
          done={view.done}
          showFinished={showFinished}
          now={now}
          actions={actions}
          state={state}
          runs={runs}
          onOpen={anchor.open}
          onLog={setLogId}
        />
        {count > 0 && (
          <AgentsFoot
            count={count}
            showFinished={showFinished}
            canArchive={view.done.length > 0}
            onToggle={() => setShowFinished(!showFinished)}
            onArchive={() => actions.archiveFinished(view.done.map((agent) => agent.id))}
          />
        )}
        {showFinished && <AgentRunsSection title="Finished runs" runs={runs.finished} boardKey={boardKey} now={now} onLog={setLogId} />}
        <ChatsList boardKey={boardKey} chats={view.chats} now={now} onPurge={onPurge} />
      </Box>
      <AgentDrawer boardKey={boardKey} id={opened?.id ?? null} agent={opened} order={order} now={now} actions={actions} onOpen={anchor.open} onClose={anchor.close} />
      <RunLogDrawer run={runs.all.find((run) => run.id === logId) ?? null} boardKey={boardKey} onClose={() => setLogId(null)} />
    </Flex>
  );
};

export { AgentsTab };
