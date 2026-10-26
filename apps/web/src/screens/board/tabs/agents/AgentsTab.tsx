/* @layer renderer-app @kind component */
import { Flex } from '@drizztdourden08/tessera';
import { useMemo, useState } from 'react';
import type { AgentsTabProps } from './AgentsTab.type';
import { useAgentActions } from './behavior/useAgentActions';
import { useAgentAnchor } from './behavior/useAgentAnchor';
import { useAgentsView } from './behavior/useAgentsView';
import { useNow } from './behavior/useNow';
import { useReconcile } from './behavior/useReconcile';
import { emptyMessage, filterAgents } from './model';
import type { AgentFilter } from './model';
import { AgentDrawer } from './sub-components/AgentDrawer';
import { AgentList } from './sub-components/AgentList';
import { AgentsHeader } from './sub-components/AgentsHeader';
import { AgentsToolbar } from './sub-components/AgentsToolbar';
import { ChatsList } from './sub-components/ChatsList';
import { ReconcilePanel } from './sub-components/ReconcilePanel';

const AgentsTab = (props: AgentsTabProps) => {
  const { boardKey, onPurge } = props;
  const view = useAgentsView(boardKey);
  const actions = useAgentActions(boardKey, view);
  const findings = useReconcile(boardKey, view.flagSignature);
  const now = useNow(view.active.length > 0);
  const [filter, setFilter] = useState<AgentFilter>('running');
  const [query, setQuery] = useState('');
  const rows = useMemo(() => filterAgents(view.active, view.done, filter, query), [view.active, view.done, filter, query]);
  const anchor = useAgentAnchor();
  const opened = [...view.active, ...view.done].find((agent) => agent.id === anchor.selected) ?? null;
  const canArchive = filter !== 'running' && view.done.length > 0;
  const empty = emptyMessage(view.active.length + view.done.length, filter);
  return (
    <Flex direction="column" gap="md">
      <AgentsHeader active={view.active} lead={view.lead} />
      <ReconcilePanel findings={findings} />
      <AgentsToolbar
        query={query}
        onQuery={setQuery}
        filter={filter}
        onFilter={setFilter}
        runningCount={view.active.length}
        doneCount={view.done.length}
        onArchive={canArchive ? () => actions.archiveFinished(view.done.map((agent) => agent.id)) : null}
      />
      <AgentList rows={rows} empty={empty} now={now} actions={actions} onOpen={anchor.open} />
      <ChatsList chats={view.chats} onPurge={onPurge} />
      <AgentDrawer id={opened?.id ?? null} agent={opened} now={now} actions={actions} onClose={anchor.close} />
    </Flex>
  );
};

export { AgentsTab };
