/* @layer renderer-app @kind component */
import { Flex, Stack } from '@drizztdourden08/tessera';
import type { BoardsListProps } from './boards-view.type';
import { BoardRow } from './sub-components/BoardRow';
import { BoardsGroup } from './sub-components/BoardsGroup';

const BoardsRows = (props: BoardsListProps) => {
  const { groups } = props;
  return (
    <Stack gap="md">
      {groups.map((group) => (
        <BoardsGroup key={group.id} group={group}>
          <Flex direction="column" gap="xs" role="list" aria-label={group.label}>
            {group.entries.map((entry) => <BoardRow key={entry.record.id} entry={entry} />)}
          </Flex>
        </BoardsGroup>
      ))}
    </Stack>
  );
};

export { BoardsRows };
