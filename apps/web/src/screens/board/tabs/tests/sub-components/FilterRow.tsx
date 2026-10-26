/* @layer renderer-app @kind component */
import { FilterBar, Flex, Toggle } from '@drizztdourden08/tessera';
import { FILTER_SWITCHES, FIND_TEXT } from '../tests.constants';
import type { FilterRowProps } from '../tests-props.type';
import { ClearFilters } from './ClearFilters';
import { FilterSelects } from './FilterSelects';

const FilterRow = (props: FilterRowProps) => {
  const { state } = props;
  const { filters, patchFilters } = state;
  const search = { searchLabel: FIND_TEXT, searchPlaceholder: FIND_TEXT, search: filters.query, onSearchChange: (query: string) => patchFilters({ query }) };
  return (
    <FilterBar
      {...search}
      extra={(
        <Flex gap="sm" align="center" wrap>
          <FilterSelects {...props} />
          {FILTER_SWITCHES.map(([field, label]) => (
            <Toggle key={field} size="sm" label={label} checked={filters[field]} onChange={(on) => patchFilters({ [field]: on })} />
          ))}
          <ClearFilters state={state} />
        </Flex>
      )}
    />
  );
};

export { FilterRow };
