/* @layer renderer-app @kind logic */
import type { TabField } from '../BoardTabsRow.type';

const resetOf = <F>(field: TabField<F>): Partial<F> => {
  if (field.kind === 'flag') return field.write(false);
  return field.kind === 'multi' ? field.write([]) : field.write('');
};

const clearPatch = <F>(fields: readonly TabField<F>[]): Partial<F> =>
  fields.reduce<Partial<F>>((patch, field) => ({ ...patch, ...resetOf(field) }), {});

export { clearPatch };
