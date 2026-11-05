/* @layer renderer-app @kind component */
import { Box, Flex, Text } from '@drizztdourden08/tessera';
import { RecordGroup } from '../../../../../shell';
import { ItemGroupSections, itemGroups, NO_ITEM_TITLE, useGroupsData } from '../../../item-groups';
import { STATUS_OPTIONS } from '../changes.constants';
import type { PrGroupsProps } from '../changes.type';
import { PrList } from './PrList';

const PrGroups = (props: PrGroupsProps) => {
  const { boardKey, prs, group } = props;
  const data = useGroupsData(boardKey);
  if (!data.ready) return <Text tone="muted">Loading the items.</Text>;
  if (group === 'none') return <PrList boardKey={boardKey} rows={prs} threads={data.threads} />;
  if (group === 'status') {
    return (
      <Flex direction="column" gap="md">
        {STATUS_OPTIONS.map(({ value, label }) => {
          const rows = prs.filter((pr) => pr.status === value);
          if (rows.length === 0) return null;
          return (
            <Box key={value} data-group={value}>
              <RecordGroup header={<Text weight={600}>{label}</Text>} count={rows.length} label={label}>
                <PrList boardKey={boardKey} rows={rows} threads={data.threads} />
              </RecordGroup>
            </Box>
          );
        })}
      </Flex>
    );
  }
  const groups = itemGroups(prs, data.items, (pr) => pr.item);
  return (
    <ItemGroupSections groups={groups} boardKey={boardKey} threads={data.threads} label="Pull requests" noneTitle={NO_ITEM_TITLE} noneFlag>
      {(rows) => <PrList boardKey={boardKey} rows={rows} threads={data.threads} />}
    </ItemGroupSections>
  );
};

export { PrGroups };
