/* @layer renderer-app @kind component */
import { Box, Disclosure, Flex } from '@drizztdourden08/tessera';
import { itemCode, RecordGroup } from '../../../../shell';
import { CLOSED_TITLE } from '../item-groups.constants';
import type { ClosedGroupsProps } from '../item-groups.type';
import { ItemGroupHeader } from './ItemGroupHeader';

const ClosedGroups = <T,>(props: ClosedGroupsProps<T>) => {
  const { groups, boardKey, threads, label, children } = props;
  if (groups.length === 0) return null;
  return (
    <Box data-group="closed">
      <Disclosure summary={`${CLOSED_TITLE} (${groups.length})`}>
        <Flex direction="column" gap="sm">
          {groups.map((group) => (
            <Box key={group.item.id} data-group={group.item.id}>
              <RecordGroup
                header={<ItemGroupHeader item={group.item} boardKey={boardKey} threads={threads} />}
                count={group.rows.length}
                defaultOpen={false}
                pinTitle={false}
                label={`${label} of ${itemCode(group.item)}`}
              >
                {children(group.rows)}
              </RecordGroup>
            </Box>
          ))}
        </Flex>
      </Disclosure>
    </Box>
  );
};

export { ClosedGroups };
