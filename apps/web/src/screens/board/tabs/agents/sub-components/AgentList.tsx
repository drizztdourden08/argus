/* @layer renderer-app @kind component */
import { EmptyState, ListItemList } from '@drizztdourden08/tessera';
import { ROW_SHAPE } from '../AgentsTab.constants';
import type { AgentListProps } from '../AgentsTab.type';
import { AgentRow } from './AgentRow';

const AgentList = (props: AgentListProps) => {
  const { rows, empty, now, actions, onOpen } = props;
  if (rows.length === 0) return <EmptyState size="sm" message={empty} />;
  return (
    <ListItemList label="Sub-agents" shape={ROW_SHAPE}>
      {rows.map((row) => <AgentRow key={row.id} id={row.id} agent={row} now={now} actions={actions} onOpen={onOpen} />)}
    </ListItemList>
  );
};

export { AgentList };
