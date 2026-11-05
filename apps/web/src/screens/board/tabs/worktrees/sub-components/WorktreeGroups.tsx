/* @layer renderer-app @kind component */
import { Box, Flex, Text } from '@drizztdourden08/tessera';
import type { Agent } from '../../../../../data/agents';
import { useCollection } from '../../../../../data/react';
import { RecordGroup } from '../../../../../shell';
import { ItemGroupSections, itemGroups, NO_ITEM_TITLE, useGroupsData } from '../../../item-groups';
import { useNow } from '../../../shared';
import { groupRows } from '../model';
import { MAIN_TITLE } from '../worktrees.constants';
import type { WorktreeGroupsProps } from '../worktrees-props.type';
import { WorktreeRecord } from './WorktreeRecord';

const WorktreeGroups = (props: WorktreeGroupsProps) => {
  const { boardKey, rows, group, meta, controls, ask, removal } = props;
  const data = useGroupsData(boardKey);
  const agents = useCollection<Agent>(boardKey, 'agents').docs;
  const now = useNow(false);
  if (!data.ready) return <Text tone="muted">Loading the items.</Text>;
  const records = (shown: typeof rows) => shown.map((row) => (
    <WorktreeRecord key={row.id} boardKey={boardKey} row={row} threads={data.threads} agents={agents} now={now} meta={meta} controls={controls} ask={ask} removal={removal} />
  ));
  if (group === 'none') return <Flex direction="column" gap="xs" role="list" aria-label="Worktrees">{records(rows)}</Flex>;
  if (group !== 'item') {
    return (
      <Flex direction="column" gap="md">
        {groupRows(rows, group, meta).map((part) => (
          <Box key={part.key} data-group={part.key || 'none'}>
            <RecordGroup header={<Text weight={600}>{part.label}</Text>} count={part.rows.length} label={part.label}>{records(part.rows)}</RecordGroup>
          </Box>
        ))}
      </Flex>
    );
  }
  const main = rows.filter((row) => row.main);
  const groups = itemGroups(rows.filter((row) => !row.main), data.items, (row) => row.item);
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
