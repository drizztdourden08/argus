/* @layer renderer-app @kind component */
import { Box, Flex, Text } from '@drizztdourden08/tessera';
import { RecordGroup } from '../../../../shell';
import type { ItemGroupSectionsProps } from '../item-groups.type';
import { ClosedGroups } from './ClosedGroups';
import { ItemGroupHeader } from './ItemGroupHeader';

const ItemGroupSections = <T,>(props: ItemGroupSectionsProps<T>) => {
  const { groups, boardKey, threads, label, noneTitle, noneFlag = false, lead, tail, children } = props;
  return (
    <Flex direction="column" gap="md">
      {lead}
      {groups.groups.map((group) => (
        <Box key={group.item.id} data-group={group.item.id}>
          <RecordGroup
            header={<ItemGroupHeader item={group.item} boardKey={boardKey} threads={threads} />}
            count={group.rows.length}
            label={`${label} of ${group.item.id}`}
          >
            {children(group.rows)}
          </RecordGroup>
        </Box>
      ))}
      {groups.none.length > 0 && (
        <Box data-group="none">
          <RecordGroup header={<Text weight={600}>{noneTitle}</Text>} count={groups.none.length} flagged={noneFlag} label={noneTitle}>
            {children(groups.none)}
          </RecordGroup>
        </Box>
      )}
      {tail}
      <ClosedGroups groups={groups.closed} boardKey={boardKey} threads={threads} label={label}>
        {children}
      </ClosedGroups>
    </Flex>
  );
};

export { ItemGroupSections };
