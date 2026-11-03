/* @layer renderer-app @kind component */
import { Button, Checkbox, Flex, SearchInput, Select } from '@drizztdourden08/tessera';
import { ANY_STATUS } from '../kanban.constants';
import type { FilterRowProps } from '../kanban-props.type';
import { threadChoices } from '../../../shared';
import { STATUS_OPTIONS, assigneeChoices, filtersActive, typeChoices } from '../model';

const FilterRow = (props: FilterRowProps) => {
  const { boardKey, state, items, threads } = props;
  const { filters, patchFilters, clearFilters } = state;
  const owners = threadChoices(threads);
  const types = typeChoices();
  const assignees = assigneeChoices(items, boardKey, threads);
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
      <Select
        width="fit"
        aria-label="Threads"
        placeholder="Threads"
        size="sm"
        multiDisplay="count"
        min={0}
        max={Math.max(owners.length, 2)}
        options={owners}
        values={filters.threads}
        onValuesChange={(values) => patchFilters({ threads: values })}
      />
      <Select
        width="fit"
        aria-label="Type"
        placeholder="Type"
        size="sm"
        multiDisplay="count"
        min={0}
        max={types.length}
        options={types}
        values={filters.types}
        onValuesChange={(values) => patchFilters({ types: values })}
      />
      <Select
        width="fit"
        aria-label="Assignee"
        placeholder="Assignee"
        size="sm"
        multiDisplay="count"
        min={0}
        max={Math.max(assignees.length, 2)}
        options={assignees}
        values={filters.assignees}
        onValuesChange={(values) => patchFilters({ assignees: values })}
      />
      <Select
        width="fit"
        aria-label="Status"
        size="sm"
        options={[{ value: ANY_STATUS, label: 'Any status' }, ...STATUS_OPTIONS]}
        value={filters.status || ANY_STATUS}
        onChange={(value) => patchFilters({ status: value === ANY_STATUS ? '' : value })}
      />
      <Checkbox size="sm" boxed label="needs you" checked={filters.needs} onChange={(needs) => patchFilters({ needs })} />
      <Checkbox size="sm" boxed label="conflict" checked={filters.conflict} onChange={(conflict) => patchFilters({ conflict })} />
      <Checkbox size="sm" boxed label="archived" checked={filters.archived} onChange={(archived) => patchFilters({ archived })} />
      {(filtersActive(filters) || filters.archived) && <Button size="sm" variant="ghost" onClick={clearFilters}>Clear filters</Button>}
    </Flex>
  );
};

export { FilterRow };
