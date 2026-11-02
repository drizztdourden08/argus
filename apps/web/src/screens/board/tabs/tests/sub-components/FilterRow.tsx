/* @layer renderer-app @kind component */
import { Checkbox, Flex, SearchInput } from '@drizztdourden08/tessera';
import { FILTER_SWITCHES, FIND_TEXT } from '../tests.constants';
import type { FilterRowProps } from '../tests-props.type';
import { ClearFilters } from './ClearFilters';
import { FilterSelects } from './FilterSelects';

const FilterRow = (props: FilterRowProps) => {
  const { state } = props;
  const { filters, patchFilters } = state;
  return (
    <Flex gap="sm" align="center" wrap>
      <SearchInput size="sm" width="narrow" aria-label={FIND_TEXT} placeholder={FIND_TEXT} value={filters.query} onChange={(query) => patchFilters({ query })} />
      <FilterSelects {...props} />
      {FILTER_SWITCHES.map(([field, label]) => (
        <Checkbox key={field} size="sm" boxed label={label} checked={filters[field]} onChange={(on) => patchFilters({ [field]: on })} />
      ))}
      <ClearFilters state={state} />
    </Flex>
  );
};

export { FilterRow };
