/* @layer renderer-app @kind component */
import { itemCode, recordMenuGroups, RecordRow, useRecordPane } from '../../../../shell';
import { assigneeOf, stageLabel, typeOf } from '../../tabs/kanban/model';
import type { ItemGroupHeaderProps } from '../item-groups.type';

const ItemRecordRow = (props: ItemGroupHeaderProps) => {
  const { item, boardKey, threads } = props;
  const pane = useRecordPane();
  const kind = typeOf(item);
  const code = itemCode(item);
  const open = () => pane.open(item.id);
  const stage = stageLabel(item, 'parent');
  return (
    <RecordRow
      head={{ kind, code, assignee: assigneeOf(item), boardKey, threads, menu: recordMenuGroups(kind, code, { open }), menuLabel: `More actions for ${code}` }}
      title={item.title}
      chips={stage ? [{ id: 'stage', text: stage, title: 'Where the item stands' }] : []}
      onOpen={open}
    />
  );
};

export { ItemRecordRow };
