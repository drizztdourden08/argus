/* @layer renderer-app @kind component */
import { Button, Checkbox, Flex, SearchInput, Select } from '@drizztdourden08/tessera';
import { ANY_STATUS, ANY_SUBJECT } from '../kanban.constants';
import type { FilterRowProps } from '../kanban-props.type';
import { threadChoices } from '../../../shared';
import { STATUS_OPTIONS, filtersActive } from '../model';

const FilterRow = (props: FilterRowProps) => {
  const { state, threads, subjects } = props;
  const { filters, patchFilters, clearFilters } = state;
  const owners = threadChoices(threads);
  return (
    <Flex gap="sm" align="center" wrap>
      <SearchInput
        size="sm"
        grow={false}
        aria-label="Search id or title"
        placeholder="Search id or title"
        value={filters.query}
        onChange={(query) => patchFilters({ query })}
      />
      <Select
        aria-label="Threads"
        placeholder="Threads"
        size="sm"
        grow={false}
        multiDisplay="count"
        min={0}
        max={Math.max(owners.length, 2)}
        options={owners}
        values={filters.threads}
        onValuesChange={(values) => patchFilters({ threads: values })}
      />
      <Select
        aria-label="Status"
        size="sm"
        grow={false}
        options={[{ value: ANY_STATUS, label: 'Any status' }, ...STATUS_OPTIONS]}
        value={filters.status || ANY_STATUS}
        onChange={(value) => patchFilters({ status: value === ANY_STATUS ? '' : value })}
      />
      <Select
        aria-label="Subject"
        size="sm"
        grow={false}
        options={[{ value: ANY_SUBJECT, label: 'Any subject' }, ...subjects.map((subject) => ({ value: subject, label: subject }))]}
        value={filters.subject || ANY_SUBJECT}
        onChange={(value) => patchFilters({ subject: value === ANY_SUBJECT ? '' : value })}
      />
      <Checkbox size="sm" label="needs you" checked={filters.needs} onChange={(needs) => patchFilters({ needs })} />
      <Checkbox size="sm" label="conflict" checked={filters.conflict} onChange={(conflict) => patchFilters({ conflict })} />
      <Checkbox size="sm" label="archived" checked={filters.archived} onChange={(archived) => patchFilters({ archived })} />
      {(filtersActive(filters) || filters.archived) && <Button size="sm" variant="ghost" onClick={clearFilters}>Clear filters</Button>}
    </Flex>
  );
};

export { FilterRow };
