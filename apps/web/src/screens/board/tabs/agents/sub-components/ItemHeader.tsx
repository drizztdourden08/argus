/* @layer renderer-app @kind component */
import { RecordLink, RecordRow, recordMenuGroups } from '../../../../../shell';
import { assigneeOf, typeOf } from '../../kanban/model';
import type { ItemHeaderProps } from '../row-props.type';

const ItemHeader = (props: ItemHeaderProps) => {
  const { boardKey, itemId, items, threads, onOpen } = props;
  const item = items.find((candidate) => candidate.id === itemId);
  if (!item) return <RecordLink id={itemId} board={boardKey} />;
  const kind = typeOf(item);
  return (
    <RecordRow
      head={{
        kind,
        code: item.id,
        assignee: assigneeOf(item),
        boardKey,
        threads,
        menu: recordMenuGroups(kind, item.id, { open: () => onOpen(item.id) }),
        menuLabel: `More actions for ${item.id}`,
      }}
      title={item.title}
      onOpen={() => onOpen(item.id)}
    />
  );
};

export { ItemHeader };
