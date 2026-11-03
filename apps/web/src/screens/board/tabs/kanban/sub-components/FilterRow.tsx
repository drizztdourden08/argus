/* @layer renderer-app @kind component */
import { Button, Flex, SearchInput } from '@drizztdourden08/tessera';
import type { FilterRowProps } from '../kanban-props.type';
import { filtersActive } from '../model';
import { FilterChecks } from './FilterChecks';
import { FilterSelects } from './FilterSelects';

const FilterRow = (props: FilterRowProps) => {
  const { state } = props;
  const { filters, patchFilters, clearFilters } = state;
  return (
    <Flex gap="sm" align="center" wrap>
      <SearchInput
        width="narrow-fill"
        size="sm"
        aria-label="Search id or title"
        placeholder="Search id or title"
        value={filters.query}
        onChange={(query) => patchFilters({ query })}
      />
      <FilterSelects {...props} />
      <FilterChecks state={state} />
      {(filtersActive(filters) || filters.archived) && <Button size="sm" variant="ghost" onClick={clearFilters}>Clear filters</Button>}
    </Flex>
  );
};

export { FilterRow };
