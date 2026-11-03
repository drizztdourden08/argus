/* @layer renderer-app @kind component */
import { recordMenuGroups, RecordRow, useRecordPane } from '../../../../shell';
import { assigneeOf, stageLabel, typeOf } from '../../tabs/kanban/model';
import type { ItemGroupHeaderProps } from '../item-groups.type';

const ItemRecordRow = (props: ItemGroupHeaderProps) => {
  const { item, boardKey, threads } = props;
  const pane = useRecordPane();
  const kind = typeOf(item);
  const open = () => pane.open(item.id);
  const stage = stageLabel(item, 'parent');
  return (
    <RecordRow
      head={{ kind, code: item.id, assignee: assigneeOf(item), boardKey, threads, menu: recordMenuGroups(kind, item.id, { open }), menuLabel: `More actions for ${item.id}` }}
      title={item.title}
      chips={stage ? [{ id: 'stage', text: stage, title: 'Where the item stands' }] : []}
      onOpen={open}
    />
  );
};

export { ItemRecordRow };
