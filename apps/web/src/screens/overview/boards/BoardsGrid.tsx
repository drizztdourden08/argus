/* @layer renderer-app @kind component */
import { Grid, Stack } from '@drizztdourden08/tessera';
import { BOARD_COLUMN_MIN } from './boards-view.constants';
import type { BoardsListProps } from './boards-view.type';
import { BoardCard } from './sub-components/BoardCard';
import { BoardsGroup } from './sub-components/BoardsGroup';

const BoardsGrid = (props: BoardsListProps) => {
  const { groups, now, send, syncs, onLog } = props;
  return (
    <Stack gap="md">
      {groups.map((group) => (
        <BoardsGroup key={group.id} group={group}>
          <Grid minColWidth={BOARD_COLUMN_MIN} gap="md">
            {group.entries.map(({ record, card, live }) => (
              <BoardCard key={record.id} record={record} card={card} live={live} now={now} send={send} sync={syncs?.[record.key] ?? null} onLog={onLog} />
            ))}
          </Grid>
        </BoardsGroup>
      ))}
    </Stack>
  );
};

export { BoardsGrid };
