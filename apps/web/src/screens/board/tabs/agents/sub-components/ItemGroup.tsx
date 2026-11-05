/* @layer renderer-app @kind component */
import { itemCode, RecordGroup } from '../../../../../shell';
import type { ItemGroupProps } from '../row-props.type';
import { AgentRow } from './AgentRow';
import { ItemHeader } from './ItemHeader';

const ItemGroup = (props: ItemGroupProps) => {
  const { itemId, items, agents, boardKey, threads, now, actions, onOpen } = props;
  const item = items.find((candidate) => candidate.id === itemId);
  return (
    <RecordGroup
      header={<ItemHeader boardKey={boardKey} itemId={itemId} items={items} threads={threads} onOpen={onOpen} />}
      count={agents.length}
      label={`Agents of ${item ? itemCode(item) : itemId}`}
    >
      {agents.map((agent) => (
        <AgentRow key={agent.id} boardKey={boardKey} id={agent.id} agent={agent} threads={threads} now={now} actions={actions} onOpen={onOpen} />
      ))}
    </RecordGroup>
  );
};

export { ItemGroup };
