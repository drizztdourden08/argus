/* @layer renderer-app @kind component */
import { Box, Flex } from '@drizztdourden08/tessera';
import { useRecordPane } from '../../../../shell';
import { FINISHED_KEY, useStoredFlag } from '../../../../stored';
import { SECOND_TICK_MS, useNow } from '../../shared';
import type { AgentsTabProps } from './AgentsTab.type';
import { useAgentActions } from './behavior/useAgentActions';
import { useAgentRuns } from './behavior/useAgentRuns';
import { useAgentsView } from './behavior/useAgentsView';
import { useReconcile } from './behavior/useReconcile';
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
  const actions = useAgentActions(boardKey, view.all);
  const findings = useReconcile(boardKey, view.flagSignature);
  const now = useNow(view.active.length > 0);
  const runs = useAgentRuns(boardKey);
  const runNow = useNow(runs.live.length > 0, SECOND_TICK_MS);
  const { open } = useRecordPane();
  const [showFinished, setShowFinished] = useStoredFlag(FINISHED_KEY);
  const count = view.finishedCount;
  return (
    <Flex direction="column" gap="md">
      <ReconcilePanel findings={findings} />
      <Box as="section" aria-label="Agents" className="agents-panel">
        <AgentsHeader active={view.active} lead={view.lead} now={now} />
        <AgentRunsSection title="Running now" runs={runs.live} boardKey={boardKey} now={runNow} />
        <AgentsBody boardKey={boardKey} view={view} showFinished={showFinished} now={now} actions={actions} onOpen={open} />
        {count > 0 && (
          <AgentsFoot
            count={count}
            showFinished={showFinished}
            canArchive={view.done.length > 0}
            onToggle={() => setShowFinished(!showFinished)}
            onArchive={() => actions.archiveFinished(view.done.map((agent) => agent.id))}
          />
        )}
        {showFinished && <AgentRunsSection title="Finished runs" runs={runs.finished} boardKey={boardKey} now={now} />}
        <ChatsList boardKey={boardKey} chats={view.chats} now={now} onPurge={onPurge} />
      </Box>
    </Flex>
  );
};

export { AgentsTab };
