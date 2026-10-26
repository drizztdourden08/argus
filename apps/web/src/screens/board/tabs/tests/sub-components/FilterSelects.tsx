/* @layer renderer-app @kind component */
import { STATUS_FILTERS } from '../model';
import { NO_BUILD_VALUE } from '../tests.constants';
import type { FilterRowProps } from '../tests-props.type';
import { ChoiceFilter } from './ChoiceFilter';
import { MultiFilter } from './MultiFilter';

const FilterSelects = (props: FilterRowProps) => {
  const { state, threads, subjects, builds, handed } = props;
  const { filters, patchFilters } = state;
  const buildOptions = [
    ...builds.map((tag) => ({ value: tag, label: tag === handed ? `${tag} · handed to you` : tag })),
    { value: NO_BUILD_VALUE, label: 'no build' },
  ];
  const buildValues = filters.nobuild ? [...filters.builds, NO_BUILD_VALUE] : filters.builds;
  const pickBuilds = (values: string[]) => patchFilters({ builds: values.filter((value) => value !== NO_BUILD_VALUE), nobuild: values.includes(NO_BUILD_VALUE) });
  return (
    <>
      <MultiFilter label="Threads" options={threads} values={filters.threads} onPick={(values) => patchFilters({ threads: values })} />
      {builds.length > 0 && <MultiFilter label="Build" options={buildOptions} values={buildValues} onPick={pickBuilds} />}
      <ChoiceFilter label="Status" any="Any status" options={STATUS_FILTERS} value={filters.status} onPick={(status) => patchFilters({ status })} />
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
