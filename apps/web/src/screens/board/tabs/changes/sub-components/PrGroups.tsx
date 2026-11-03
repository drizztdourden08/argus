/* @layer renderer-app @kind component */
import { Flex, Text } from '@drizztdourden08/tessera';
import { ItemGroupSections, itemGroups, NO_ITEM_TITLE, useGroupsData } from '../../../item-groups';
import type { PrGroupsProps } from '../changes.type';
import { sortPrs } from '../model';
import { PrCard } from './PrCard';

const PrGroups = (props: PrGroupsProps) => {
  const { boardKey, prs } = props;
  const data = useGroupsData(boardKey);
  if (!data.ready) return <Text tone="muted">Loading the items.</Text>;
  const groups = itemGroups(sortPrs(prs), data.items, (pr) => pr.item);
  return (
    <ItemGroupSections groups={groups} boardKey={boardKey} threads={data.threads} label="Pull requests" noneTitle={NO_ITEM_TITLE} noneFlag>
      {(rows) => (
        <Flex direction="column" gap="sm" role="list" aria-label="Pull requests">
          {rows.map((pr) => <PrCard key={pr.id} boardKey={boardKey} pr={pr} threads={data.threads} />)}
        </Flex>
      )}
    </ItemGroupSections>
  );
};

export { PrGroups };
