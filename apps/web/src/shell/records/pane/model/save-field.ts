/* @layer renderer-app @kind logic */
import type { FieldValue } from '../../../../data/kanban';
import type { FieldProps } from '../pane.type';

const saveField = (props: Pick<FieldProps, 'name' | 'item' | 'ops'>, value: unknown): void => {
  const { name, item, ops } = props;
  void ops.setField(item, `fields.${name}`, value as FieldValue);
};

export { saveField };
