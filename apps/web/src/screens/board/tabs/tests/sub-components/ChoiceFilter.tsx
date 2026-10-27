/* @layer renderer-app @kind component */
import { Select } from '@drizztdourden08/tessera';
import { ANY_CHOICE } from '../tests.constants';
import type { ChoiceFilterProps } from '../tests-props.type';

const ChoiceFilter = (props: ChoiceFilterProps) => {
  const { label, any, options, value, onPick } = props;
  return (
    <Select
      size="sm"
      width="fit"
      aria-label={label}
      options={[{ value: ANY_CHOICE, label: any }, ...options]}
      value={value || ANY_CHOICE}
      onChange={(next) => onPick(next === ANY_CHOICE ? '' : next)}
    />
  );
};

export { ChoiceFilter };
