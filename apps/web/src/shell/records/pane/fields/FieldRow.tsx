/* @layer renderer-app @kind component */
import type { FieldProps } from './fields.type';
import { FieldEdit } from './FieldEdit';
import { FieldFrame } from './FieldFrame';

const FieldRow = (props: FieldProps) => (
  <FieldFrame name={props.name}>
    <FieldEdit {...props} />
  </FieldFrame>
);

export { FieldRow };
