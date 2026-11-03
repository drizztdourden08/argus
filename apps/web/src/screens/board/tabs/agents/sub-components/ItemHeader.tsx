/* @layer renderer-app @kind component */
import { itemCode, RecordGroupHeader, RecordLink } from '../../../../../shell';
import { typeOf } from '../../kanban/model';
import type { ItemHeaderProps } from '../row-props.type';

const ItemHeader = (props: ItemHeaderProps) => {
  const { boardKey, itemId, items, onOpen } = props;
  const item = items.find((candidate) => candidate.id === itemId);
  if (!item) return <RecordLink id={itemId} board={boardKey} />;
  const code = itemCode(item);
  return <RecordGroupHeader kind={typeOf(item)} code={code} title={item.title} boardKey={boardKey} onOpen={() => onOpen(code)} />;
};

export { ItemHeader };
