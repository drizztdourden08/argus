/* @layer renderer-app @kind component */
import { ListItemList, Stack } from '@drizztdourden08/tessera';
import { ROW_SHAPE } from './boards-view.constants';
import type { BoardsListProps } from './boards-view.type';
import { BoardRow } from './sub-components/BoardRow';
import { BoardsGroup } from './sub-components/BoardsGroup';

const BoardsRows = (props: BoardsListProps) => {
  const { groups } = props;
  return (
    <Stack gap="md">
      {groups.map((group) => (
        <BoardsGroup key={group.id} group={group}>
          <ListItemList label={group.label} shape={ROW_SHAPE}>
            {group.entries.map((entry) => <BoardRow key={entry.record.id} entry={entry} />)}
          </ListItemList>
        </BoardsGroup>
      ))}
    </Stack>
  );
};

export { BoardsRows };
