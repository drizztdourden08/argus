/* @layer renderer-app @kind component */
import { Inline, Tag } from '@drizztdourden08/tessera';
import { CHIP_ALL, CHIP_LOOK } from '../overview.constants';
import type { ChipFilterProps } from '../overview.type';

const ChipFilter = (props: ChipFilterProps) => {
  const { label, value, all, options, onChange } = props;
  const current = value || CHIP_ALL;
  const chips = [{ value: CHIP_ALL, label: all }, ...options];
  return (
    <Inline gap="xs" align="center" wrap role="radiogroup" aria-label={label}>
      {chips.map((chip) => (
        <Tag
          key={chip.value}
          {...CHIP_LOOK}
          pill
          size="lg"
          selectedTone="neutral"
          role="radio"
          selected={chip.value === current}
          onSelect={() => onChange(chip.value === current || chip.value === CHIP_ALL ? '' : chip.value)}
        >
          {chip.label}
        </Tag>
      ))}
    </Inline>
  );
};

export { ChipFilter };
