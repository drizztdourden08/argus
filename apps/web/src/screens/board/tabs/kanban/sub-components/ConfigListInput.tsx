/* @layer renderer-app @kind component */
import { Field, TextInput } from '@drizztdourden08/tessera';
import { configParseList } from '../model';
import type { ConfigListInputProps } from '../config-editor.type';

const ConfigListInput = (props: ConfigListInputProps) => {
  const { label, value, hint, onCommit } = props;
  return (
    <Field label={label} hint={hint}>
      <TextInput key={value.join(',')} defaultValue={value.join(', ')} onBlur={(event) => onCommit(configParseList(event.currentTarget.value))} />
    </Field>
  );
};

export { ConfigListInput };
