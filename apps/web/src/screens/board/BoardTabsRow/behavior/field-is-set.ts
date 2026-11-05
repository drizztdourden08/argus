/* @layer renderer-app @kind logic */
import type { TabField } from '../BoardTabsRow.type';

const fieldIsSet = <F>(field: TabField<F>, filters: F): boolean => {
  if (field.kind === 'flag') return field.read(filters);
  if (field.kind === 'multi') return field.read(filters).length > 0;
  return field.read(filters) !== '';
};

export { fieldIsSet };
