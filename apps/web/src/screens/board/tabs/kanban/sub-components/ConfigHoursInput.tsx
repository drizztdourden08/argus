/* @layer renderer-app @kind component */
import { Field, NumberInput } from '@drizztdourden08/tessera';
import { HOUR_MS } from '../model';
import type { ConfigHoursInputProps } from '../config-editor.type';

const ConfigHoursInput = (props: ConfigHoursInputProps) => {
  const { label, ms, onChange } = props;
  return (
    <Field label={label} width="sm">
      <NumberInput min={0} step={1} value={ms / HOUR_MS} onChange={(hours) => onChange(Math.max(0, Math.round(hours * HOUR_MS)))} />
    </Field>
  );
};

export { ConfigHoursInput };
