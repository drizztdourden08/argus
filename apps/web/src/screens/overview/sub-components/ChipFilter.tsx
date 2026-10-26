/* @layer renderer-app @kind component */
import { TagPicker } from '@drizztdourden08/tessera';
import { CHIP_ALL, CHIP_LOOK } from '../overview.constants';
import type { ChipFilterProps } from '../overview.type';

const ChipFilter = (props: ChipFilterProps) => {
  const { label, value, all, options, onChange } = props;
  const chips = [{ value: CHIP_ALL, label: all }, ...options].map((option) => ({ ...CHIP_LOOK, ...option }));
  return (
    <TagPicker
      aria-label={label}
      single
      value={[value || CHIP_ALL]}
      groups={[{ id: label, options: chips }]}
      onChange={(next) => onChange(next[0] === undefined || next[0] === CHIP_ALL ? '' : next[0])}
    />
  );
};

export { ChipFilter };
