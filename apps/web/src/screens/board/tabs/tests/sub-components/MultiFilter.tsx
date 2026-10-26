/* @layer renderer-app @kind component */
import { Select } from '@drizztdourden08/tessera';
import type { MultiFilterProps } from '../tests-props.type';

const MultiFilter = (props: MultiFilterProps) => {
  const { label, options, values, onPick } = props;
  return (
    <Select
      size="sm"
      aria-label={label}
      placeholder={label}
      multiDisplay="count"
      options={options}
      values={values}
      min={0}
      max={Math.max(options.length, 2)}
      onValuesChange={onPick}
    />
  );
};

export { MultiFilter };
