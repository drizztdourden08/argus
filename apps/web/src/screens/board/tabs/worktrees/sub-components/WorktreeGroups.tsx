/* @layer renderer-app @kind component */
import { Box, Text } from '@drizztdourden08/tessera';
import { RecordGroup } from '../../../../../shell';
import { ItemGroupSections, itemGroups, NO_ITEM_TITLE, useGroupsData } from '../../../item-groups';
import { useNow } from '../../../shared';
import { MAIN_TITLE } from '../worktrees.constants';
import type { WorktreeGroupsProps } from '../worktrees-props.type';
import { WorktreeRecord } from './WorktreeRecord';

const WorktreeGroups = (props: WorktreeGroupsProps) => {
  const { boardKey, rows, meta, controls, ask, removal } = props;
  const data = useGroupsData(boardKey);
  const now = useNow(false);
  if (!data.ready) return <Text tone="muted">Loading the items.</Text>;
  const main = rows.filter((row) => row.main);
  const groups = itemGroups(rows.filter((row) => !row.main), data.items, (row) => row.item);
  const records = (shown: typeof rows) => shown.map((row) => (
    <WorktreeRecord key={row.id} boardKey={boardKey} row={row} threads={data.threads} now={now} meta={meta} controls={controls} ask={ask} removal={removal} />
  ));
  return (
    <ItemGroupSections
      groups={groups}
      boardKey={boardKey}
      threads={data.threads}
      label="Worktrees"
      noneTitle={NO_ITEM_TITLE}
      noneFlag
      tail={main.length > 0 && (
        <Box data-group="main">
          <RecordGroup header={<Text weight={600}>{MAIN_TITLE}</Text>} count={main.length} label={MAIN_TITLE}>{records(main)}</RecordGroup>
        </Box>
      )}
    >
      {records}
    </ItemGroupSections>
  );
};

export { WorktreeGroups };
