/* @layer renderer-app @kind component */
import { Button, Flex, Select, Text } from '@drizztdourden08/tessera';
import type { SelectionBarProps } from '../kanban-props.type';
import type { BulkTarget } from '../kanban.type';
import { threadChoices } from '../../../shared';
import { STATUS_OPTIONS } from '../model';

const SelectionBar = (props: SelectionBarProps) => {
  const { items, threads, selection, ops } = props;
  const picked = items.filter((item) => selection.has(item.id));
  if (picked.length === 0) return null;
  const move = (to: BulkTarget) => {
    ops.move(picked, to);
    selection.clear();
  };
  const soft = (kind: 'archive' | 'delete') => {
    ops.soft(kind, picked);
    selection.clear();
  };
  return (
    <Flex gap="sm" align="center" wrap role="toolbar" aria-label="Selected items">
      <Text>{`${picked.length} selected`}</Text>
      <Select aria-label="Move selected to status" placeholder="Move to status" size="sm" options={STATUS_OPTIONS} onChange={(status) => move({ status })} />
      <Select aria-label="Reassign selected to thread" placeholder="Reassign to thread" size="sm" options={threadChoices(threads)} onChange={(thread) => move({ thread })} />
      <Button size="sm" onClick={() => soft('archive')}>Archive</Button>
      <Button size="sm" variant="danger" onClick={() => soft('delete')}>Delete</Button>
      <Button size="sm" variant="ghost" onClick={selection.clear}>Clear selection</Button>
    </Flex>
  );
};

export { SelectionBar };
