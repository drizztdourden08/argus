/* @layer renderer-app @kind component */
import { Select } from '@drizztdourden08/tessera';
import type { FieldSelectProps } from './fields.type';

const FieldSelect = (props: FieldSelectProps) => {
  const { label, options, value, onPick } = props;
  return <Select aria-label={label} size="sm" width="fit" options={options} value={value} onChange={(next) => next !== value && onPick(next)} />;
};

export { FieldSelect };
