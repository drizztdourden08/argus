/* @layer renderer-app @kind component */
import { FilterBar } from '@drizztdourden08/tessera';
import { useMemo } from 'react';
import { BoardFocus, useFocusActive } from '../../BoardFocus';
import { fieldIsSet } from '../behavior/field-is-set';
import { fieldsSchema } from '../behavior/fields-schema';
import { useFieldClauses } from '../behavior/useFieldClauses';
import type { TabFilterBarProps } from '../BoardTabsRow.type';
import { TabFilterRow } from './TabFilterRow';

const TabFilterBar = <F extends { query: string }>(props: TabFilterBarProps<F>) => {
  const { tab, boardKey, fields, filters, patchFilters, placeholder, focus = true } = props;
  const focused = useFocusActive(boardKey) && focus;
  const count = fields.filter((field) => fieldIsSet(field, filters)).length + (filters.query ? 1 : 0) + (focused ? 1 : 0);
  const schema = useMemo(() => fieldsSchema(fields), [fields]);
  const { clauses, onChange } = useFieldClauses(fields, filters, patchFilters);
  return (
    <TabFilterRow tab={tab} count={count}>
      <FilterBar
        search={filters.query}
        onSearchChange={(query) => patchFilters({ query } as Partial<F>)}
        searchPlaceholder={placeholder}
        searchLabel={placeholder}
        leading={focus ? <BoardFocus boardKey={boardKey} /> : undefined}
        schema={schema}
        clauses={clauses}
        onChange={onChange}
      />
    </TabFilterRow>
  );
};

export { TabFilterBar };
