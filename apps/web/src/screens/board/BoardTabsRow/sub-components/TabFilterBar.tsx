/* @layer renderer-app @kind component */
import { FilterBar } from '@drizztdourden08/tessera';
import { BoardFocus, useFocusActive } from '../../BoardFocus';
import { fieldIsSet } from '../behavior/field-is-set';
import type { TabFilterBarProps } from '../BoardTabsRow.type';
import { TabFieldControls } from './TabFieldControls';
import { TabFilterRow } from './TabFilterRow';

const TabFilterBar = <F extends { query: string }>(props: TabFilterBarProps<F>) => {
  const { tab, boardKey, fields, filters, patchFilters, placeholder } = props;
  const focused = useFocusActive(boardKey);
  const count = fields.filter((field) => fieldIsSet(field, filters)).length + (filters.query ? 1 : 0) + (focused ? 1 : 0);
  return (
    <TabFilterRow tab={tab} count={count}>
      <FilterBar
        search={filters.query}
        onSearchChange={(query) => patchFilters({ query } as Partial<F>)}
        searchPlaceholder={placeholder}
        searchLabel={placeholder}
        leading={<BoardFocus boardKey={boardKey} />}
        extra={<TabFieldControls fields={fields} filters={filters} patch={patchFilters} />}
      />
    </TabFilterRow>
  );
};

export { TabFilterBar };
