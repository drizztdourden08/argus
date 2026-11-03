/* @layer renderer-app @kind component */
import { Select } from '@drizztdourden08/tessera';
import { ANY_STATUS } from '../kanban.constants';
import type { FilterRowProps } from '../kanban-props.type';
import { threadChoices } from '../../../shared';
import { STATUS_OPTIONS, assigneeChoices, typeChoices } from '../model';

const FilterSelects = (props: FilterRowProps) => {
  const { boardKey, state, items, threads } = props;
  const { filters, patchFilters } = state;
  const owners = threadChoices(threads);
  const types = typeChoices();
  const assignees = assigneeChoices(items, boardKey, threads);
  return (
    <>
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
    </>
  );
};

export { FilterSelects };
