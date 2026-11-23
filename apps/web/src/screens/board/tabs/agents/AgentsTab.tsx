/* @layer renderer-app @kind component */
import { Flex } from '@drizztdourden08/tessera';
import { useState } from 'react';
import { useRecordPane } from '../../../../shell';
import { FINISHED_KEY, useStoredFlag } from '../../../../stored';
import { useTabView } from '../../BoardTabsRow';
import { SECOND_TICK_MS, TabPanel, useNow, useTabLayout } from '../../shared';
import type { AgentsTabProps } from './AgentsTab.type';
import { useAgentActions } from './behavior/useAgentActions';
import { useAgentRuns } from './behavior/useAgentRuns';
import { useAgentsView } from './behavior/useAgentsView';
import { useReconcile } from './behavior/useReconcile';
import { AGENT_FILTERS, AGENT_GROUP_IDS, AGENT_ORDER_IDS, AGENTS_TAB, LAYOUT_IDS, agentStats, allOpen, setRowOpen, toggleAll } from './model';
import { AgentRunsSection } from './sub-components/AgentRunsSection';
import { AgentsBody } from './sub-components/AgentsBody';
import { AgentsFoot } from './sub-components/AgentsFoot';
import { ReconcilePanel } from './sub-components/ReconcilePanel';
import { AgentsTools } from './AgentsTools';
import './AgentsTab.css';

const AgentsTab = (props: AgentsTabProps) => {
  const { boardKey } = props;
  const [layout, setLayout] = useTabLayout(AGENTS_TAB, boardKey, LAYOUT_IDS, 'list');
  const tabView = useTabView(AGENTS_TAB, { groups: AGENT_GROUP_IDS, orders: AGENT_ORDER_IDS, group: 'none', order: 'updated', direction: 'desc', filters: AGENT_FILTERS });
  const view = useAgentsView(boardKey, { filters: tabView.filters, order: tabView.order, direction: tabView.direction });
  const actions = useAgentActions(boardKey, view.all);
  const findings = useReconcile(boardKey, view.flagSignature);
  const now = useNow(view.active.length > 0);
  const runs = useAgentRuns(boardKey);
  const runNow = useNow(runs.live.length > 0, SECOND_TICK_MS);
  const { open: openRecord } = useRecordPane();
  const [showFinished, setShowFinished] = useStoredFlag(FINISHED_KEY);
  const count = view.finishedCount;
  const [open, setOpen] = useState<ReadonlySet<string>>(new Set());
  const ids = [...view.shown.active, ...(showFinished ? view.shown.done : [])].map((agent) => agent.id);
  const expand = { all: allOpen(open, ids), onToggle: () => setOpen(toggleAll(open, ids)) };
  const finished = { count, shown: showFinished, onToggle: () => setShowFinished(!showFinished) };
  return (
    <Flex direction="column" gap="md">
      <AgentsTools boardKey={boardKey} tabView={tabView} view={view} finished={finished} layout={{ value: layout, onChange: setLayout }} expand={expand} />
      <ReconcilePanel findings={findings} />
      <TabPanel label="Agents" className="agents-panel" stats={agentStats(view.active, count)}>
        <AgentRunsSection title="Running now" runs={runs.live} boardKey={boardKey} now={runNow} />
        <AgentsBody boardKey={boardKey} view={view} showFinished={showFinished} group={tabView.group} layout={layout} now={now} actions={actions} onOpen={openRecord} open={open} onRowOpen={(id, value) => setOpen(setRowOpen(open, id, value))} />
        {showFinished && count > 0 && (
          <AgentsFoot canArchive={view.shown.done.length > 0} onArchive={() => actions.archiveFinished(view.shown.done.map((agent) => agent.id))} />
        )}
        {showFinished && <AgentRunsSection title="Finished runs" runs={runs.finished} boardKey={boardKey} now={now} />}
      </TabPanel>
    </Flex>
  );
};

export { AgentsTab };
