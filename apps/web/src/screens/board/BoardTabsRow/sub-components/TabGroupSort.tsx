/* @layer renderer-app @kind component */
import { Flex, GroupControl, SortControl } from '@drizztdourden08/tessera';
import type { SortState } from '@drizztdourden08/tessera';
import { pickChoice } from '../behavior/pick-choice';
import type { TabGroupSortProps } from '../BoardTabsRow.type';

const TabGroupSort = <G extends string, O extends string>({ group, sort }: TabGroupSortProps<G, O>) => {
  const pickGroup = (id: string | null) => {
    const next = group && pickChoice(group.options, id);
    if (next && group) group.onChange(next);
  };
  const pickSort = (change: SortState) => {
    const next = pickChoice(sort.options, change.key);
    if (next) sort.onChange(next, change.direction);
  };
  return (
    <Flex inline gap="xs" align="center">
      {group && <GroupControl options={group.options} value={group.value} onChange={pickGroup} />}
      <SortControl options={sort.options} value={{ key: sort.order, direction: sort.direction }} onChange={pickSort} />
    </Flex>
  );
};

export { TabGroupSort };
