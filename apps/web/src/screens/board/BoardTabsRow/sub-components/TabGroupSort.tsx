/* @layer renderer-app @kind component */
import { Flex, GroupBar, SortControl } from '@drizztdourden08/tessera';
import type { SortState } from '@drizztdourden08/tessera';
import { pickChoice } from '../behavior/pick-choice';
import type { TabGroupSortProps } from '../BoardTabsRow.type';

const TabGroupSort = <G extends string, O extends string>({ group, sort }: TabGroupSortProps<G, O>) => {
  const pickGroup = (ids: string[]) => {
    if (!group) return;
    const next = ids.length === 0 ? group.empty : pickChoice(group.options, ids.at(-1) ?? null);
    if (next) group.onChange(next);
  };
  const pickSort = (change: SortState) => {
    const next = pickChoice(sort.options, change.key);
    if (next) sort.onChange(next, change.direction);
  };
  return (
    <Flex inline gap="xs" align="center">
      {group && (
        <GroupBar
          options={group.options.filter((option) => option.id !== group.empty)}
          value={group.value === group.empty ? [] : [group.value]}
          maxLevels={1}
          onChange={pickGroup}
        />
      )}
      <SortControl options={sort.options} value={{ key: sort.order, direction: sort.direction }} onChange={pickSort} />
    </Flex>
  );
};

export { TabGroupSort };
