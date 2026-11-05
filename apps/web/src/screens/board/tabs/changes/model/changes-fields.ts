/* @layer renderer-app @kind logic */
import type { ItemRef } from '../../../../../data/kanban';
import { itemCode } from '../../../../../shell';
import type { TabField } from '../../../BoardTabsRow';
import type { Option } from '../../../shared';
import { STATUS_OPTIONS } from '../changes.constants';
import type { ChangesFilters, PullRequestDoc } from '../changes.type';

type FieldPath = keyof Omit<ChangesFilters, 'query'>;

const distinct = (values: readonly (string | undefined)[]): Option[] =>
  [...new Set(values.filter((value): value is string => Boolean(value)))].sort().map((value) => ({ value, label: value }));

const single = (path: FieldPath, label: string, options: readonly Option[]): TabField<ChangesFilters> => ({
  kind: 'single',
  path,
  label,
  options,
  read: (filters) => filters[path],
  write: (value) => ({ [path]: value }),
});

const changesFields = (prs: readonly PullRequestDoc[], items: readonly ItemRef[]): TabField<ChangesFilters>[] => [
  single('status', 'Status', STATUS_OPTIONS),
  single('item', 'Item', items.filter((item) => !item.deletedAt).map((item) => ({ value: item.id, label: `${itemCode(item)} ${item.title}` }))),
  single('author', 'Author', distinct(prs.map((pr) => pr.createdBy))),
  single('base', 'Base', distinct(prs.map((pr) => pr.base))),
];

export { changesFields };
