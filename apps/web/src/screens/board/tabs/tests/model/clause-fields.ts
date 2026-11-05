/* @layer renderer-app @kind logic */
import type { ClauseField } from '../../../BoardTabsRow';
import type { Option } from '../../../shared';
import { NO_BUILD_VALUE } from '../tests.constants';
import { STATUS_FILTERS } from './model.constants';
import type { TestFilters } from './model.type';

interface FieldInput {
  threads: Option[];
  subjects: string[];
  plans: Option[];
  builds: string[];
  handed: string | null;
}

type Field = ClauseField<TestFilters>;

const buildField = (builds: string[], handed: string | null): Field => ({
  kind: 'multi',
  path: 'builds',
  label: 'Build',
  options: [...builds.map((tag) => ({ value: tag, label: tag === handed ? `${tag} · handed to you` : tag })), { value: NO_BUILD_VALUE, label: 'no build' }],
  read: (filters) => (filters.nobuild ? [...filters.builds, NO_BUILD_VALUE] : filters.builds),
  write: (values) => ({ builds: values.filter((value) => value !== NO_BUILD_VALUE), nobuild: values.includes(NO_BUILD_VALUE) }),
});

const planField = (plans: Option[]): Field => ({ kind: 'single', path: 'plan', label: 'Plan', options: plans, read: (filters) => filters.plan, write: (plan) => ({ plan }) });

const testsFields = (input: FieldInput): Field[] => [
  { kind: 'multi', path: 'threads', label: 'Threads', options: input.threads, read: (filters) => filters.threads, write: (values) => ({ threads: values }) },
  ...(input.builds.length > 0 ? [buildField(input.builds, input.handed)] : []),
  { kind: 'single', path: 'status', label: 'Status', options: STATUS_FILTERS, read: (filters) => filters.status, write: (status) => ({ status }) },
  ...(input.plans.length > 0 ? [planField(input.plans)] : []),
  {
    kind: 'single',
    path: 'subject',
    label: 'Subject',
    options: input.subjects.map((subject) => ({ value: subject, label: subject })),
    read: (filters) => filters.subject,
    write: (subject) => ({ subject }),
  },
  { kind: 'flag', path: 'needs', label: 'Needs you', read: (filters) => filters.needs, write: (needs) => ({ needs }) },
  { kind: 'flag', path: 'conflict', label: 'Conflict', read: (filters) => filters.conflict, write: (conflict) => ({ conflict }) },
  { kind: 'flag', path: 'archived', label: 'Archived', read: (filters) => filters.archived, write: (archived) => ({ archived }) },
];

export { testsFields };
