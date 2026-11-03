/* @layer renderer-app @kind component */
import { fieldOf } from '../../../../data/registry';
import type { FieldProps } from '../pane.type';
import { FieldView } from './FieldView';
import { FIELD_EDITORS } from './fields.constants';

const FieldEdit = (props: FieldProps) => {
  const spec = fieldOf(props.name);
  const Editor = spec ? FIELD_EDITORS[spec.content] : undefined;
  return Editor ? <Editor {...props} /> : <FieldView {...props} />;
};

export { FieldEdit };
