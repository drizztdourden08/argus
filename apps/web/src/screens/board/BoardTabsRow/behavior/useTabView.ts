/* @layer renderer-app @kind logic */
import type { SortDirection } from '@drizztdourden08/tessera';
import { useStored } from '../../shared';
import type { TabView, TabViewSetup } from '../BoardTabsRow.type';
import { parseFilters } from './parse-filters';
import { parseSort } from './parse-sort';
import { viewKey } from './view-key';

const useTabView = <G extends string, O extends string, F extends object>(tab: string, setup: TabViewSetup<G, O, F>): TabView<G, O, F> => {
  const [storedGroup, writeGroup] = useStored(viewKey(tab, 'group'));
  const [storedSort, writeSort] = useStored(viewKey(tab, 'sort'));
  const [storedFilters, writeFilters] = useStored(viewKey(tab, 'filters'));
  const group = setup.groups.find((candidate) => candidate === storedGroup) ?? setup.group;
  const sort = parseSort(storedSort, setup.orders, { order: setup.order, direction: setup.direction });
  const filters = parseFilters(storedFilters, setup.filters);
  return {
    group,
    setGroup: writeGroup,
    order: sort.order,
    direction: sort.direction,
    setSort: (order: O, direction: SortDirection) => writeSort(`${order}:${direction}`),
    filters,
    patchFilters: (values) => writeFilters(JSON.stringify({ ...filters, ...values })),
  };
};

export { useTabView };
