/* @layer renderer-app @kind component */
import { itemCode, RecordGroup } from '../../../../../shell';
import type { ItemGroupProps } from '../row-props.type';
import { AgentCells } from './AgentCells';
import { ItemHeader } from './ItemHeader';

const ItemGroup = (props: ItemGroupProps) => {
  const { itemId, items, agents, ...row } = props;
  const { boardKey, threads, onOpen } = row;
  const item = items.find((candidate) => candidate.id === itemId);
  return (
    <RecordGroup
      header={<ItemHeader boardKey={boardKey} itemId={itemId} items={items} threads={threads} onOpen={onOpen} />}
      count={agents.length}
      label={`Agents of ${item ? itemCode(item) : itemId}`}
    >
      <AgentCells agents={agents} {...row} />
    </RecordGroup>
  );
};

export { ItemGroup };
