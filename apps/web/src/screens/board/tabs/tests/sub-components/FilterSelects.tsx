/* @layer renderer-app @kind component */
import { STATUS_FILTERS } from '../model';
import type { FilterRowProps } from '../tests-props.type';
import { ChoiceFilter } from './ChoiceFilter';
import { MultiFilter } from './MultiFilter';

const FilterSelects = (props: FilterRowProps) => {
  const { state, threads, subjects, plans } = props;
  const { filters, patchFilters } = state;
  return (
    <>
      <MultiFilter label="Threads" options={threads} values={filters.threads} onPick={(values) => patchFilters({ threads: values })} />
      <ChoiceFilter label="Status" any="Any status" options={STATUS_FILTERS} value={filters.status} onPick={(status) => patchFilters({ status })} />
      {plans.length > 0 && <ChoiceFilter label="Plan" any="Any plan" options={plans} value={filters.plan} onPick={(plan) => patchFilters({ plan })} />}
      <ChoiceFilter
        label="Subject"
        any="Any subject"
        options={subjects.map((subject) => ({ value: subject, label: subject }))}
        value={filters.subject}
        onPick={(subject) => patchFilters({ subject })}
      />
    </>
  );
};

export { FilterSelects };
