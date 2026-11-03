/* @layer renderer-app @kind component */
import { Checkbox } from '@drizztdourden08/tessera';
import type { FilterChecksProps } from '../kanban-props.type';

const FilterChecks = ({ state }: FilterChecksProps) => {
  const { filters, patchFilters } = state;
  return (
    <>
      <Checkbox size="sm" boxed label="needs you" checked={filters.needs} onChange={(needs) => patchFilters({ needs })} />
      <Checkbox size="sm" boxed label="conflict" checked={filters.conflict} onChange={(conflict) => patchFilters({ conflict })} />
      <Checkbox size="sm" boxed label="archived" checked={filters.archived} onChange={(archived) => patchFilters({ archived })} />
    </>
  );
};

export { FilterChecks };
