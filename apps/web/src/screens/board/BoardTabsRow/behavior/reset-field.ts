/* @layer renderer-app @kind logic */
import type { TabField } from '../BoardTabsRow.type';

const resetField = <F>(field: TabField<F>): Partial<F> => {
  if (field.kind === 'flag') return field.write(false);
  return field.kind === 'multi' ? field.write([]) : field.write('');
};

export { resetField };
