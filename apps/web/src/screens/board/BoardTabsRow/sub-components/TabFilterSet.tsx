/* @layer renderer-app @kind component */
import { useFocusActive } from '../../BoardFocus';
import { useClauseFilters } from '../behavior/useClauseFilters';
import type { TabFilterSetProps } from '../BoardTabsRow.type';
import { ClauseFilterRow } from './ClauseFilterRow';
import { TabFilters } from './TabFilters';

const TabFilterSet = <F extends { query: string }>(props: TabFilterSetProps<F>) => {
  const { tab, boardKey, fields, filters, patchFilters, placeholder } = props;
  const bar = useClauseFilters(fields, filters, patchFilters);
  const focused = useFocusActive(boardKey);
  const count = bar.count + (filters.query ? 1 : 0) + (focused ? 1 : 0);
  return (
    <TabFilters tab={tab} count={count}>
      <ClauseFilterRow boardKey={boardKey} search={filters.query} placeholder={placeholder} onSearch={(query) => patchFilters({ query } as Partial<F>)} bar={bar} />
    </TabFilters>
  );
};

export { TabFilterSet };
