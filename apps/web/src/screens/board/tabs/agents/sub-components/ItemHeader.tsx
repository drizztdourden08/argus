/* @layer renderer-app @kind component */
import { itemCode, RecordLink, RecordRow, recordMenuGroups } from '../../../../../shell';
import { assigneeOf, typeOf } from '../../kanban/model';
import type { ItemHeaderProps } from '../row-props.type';

const ItemHeader = (props: ItemHeaderProps) => {
  const { boardKey, itemId, items, threads, onOpen } = props;
  const item = items.find((candidate) => candidate.id === itemId);
  if (!item) return <RecordLink id={itemId} board={boardKey} />;
  const kind = typeOf(item);
  const code = itemCode(item);
  return (
    <RecordRow
      head={{
        kind,
        code,
        assignee: assigneeOf(item),
        boardKey,
        threads,
        menu: recordMenuGroups(kind, code, { open: () => onOpen(code) }),
        menuLabel: `More actions for ${code}`,
      }}
      title={item.title}
      onOpen={() => onOpen(code)}
    />
  );
};

export { ItemHeader };
