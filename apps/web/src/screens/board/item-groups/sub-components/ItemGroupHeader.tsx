/* @layer renderer-app @kind component */
import { itemCode, RecordGroupHeader, useRecordPane } from '../../../../shell';
import { stageLabel, typeOf } from '../../tabs/kanban/model';
import type { ItemGroupHeaderProps } from '../item-groups.type';

const ItemGroupHeader = (props: ItemGroupHeaderProps) => {
  const { item, boardKey } = props;
  const pane = useRecordPane();
  const code = itemCode(item);
  const stage = stageLabel(item, 'parent');
  return (
    <RecordGroupHeader
      kind={typeOf(item)}
      code={code}
      title={item.title}
      boardKey={boardKey}
      onOpen={() => pane.open(code)}
      tag={stage ? { text: stage, title: 'Where the item stands' } : undefined}
    />
  );
};

export { ItemGroupHeader };
