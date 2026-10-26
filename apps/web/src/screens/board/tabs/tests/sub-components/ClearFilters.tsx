/* @layer renderer-app @kind component */
import { Button } from '@drizztdourden08/tessera';
import { filtersActive } from '../model';
import type { ClearFiltersProps } from '../tests-props.type';

const ClearFilters = (props: ClearFiltersProps) => {
  const { filters, clearFilters } = props.state;
  if (!filtersActive(filters) && !filters.archived) return null;
  return <Button size="sm" variant="ghost" onClick={clearFilters}>Clear filters</Button>;
};

export { ClearFilters };
