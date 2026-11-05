/* @layer renderer-app @kind logic */
import { flagFields } from '../../../BoardTabsRow';
import type { ClauseField } from '../../../BoardTabsRow';
import type { Option } from '../../../shared';
import { STATUS_FILTERS } from './model.constants';
import type { TestFilters } from './model.type';

interface FieldInput {
  threads: Option[];
  subjects: string[];
  plans: Option[];
}

type Field = ClauseField<TestFilters>;

const planField = (plans: Option[]): Field => ({ kind: 'single', path: 'plan', label: 'Plan', options: plans, read: (filters) => filters.plan, write: (plan) => ({ plan }) });

const testsFields = (input: FieldInput): Field[] => [
  { kind: 'multi', path: 'threads', label: 'Threads', options: input.threads, read: (filters) => filters.threads, write: (values) => ({ threads: values }) },
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
  ...flagFields<TestFilters>(),
];

export { testsFields };
