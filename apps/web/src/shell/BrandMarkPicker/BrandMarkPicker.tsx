/* @layer renderer-app @kind component */
import { Flex, Pressable } from '@drizztdourden08/tessera';
import { BRAND_FAMILY, BrandMark } from '@drizztdourden08/tessera/brand';
import { AVATAR_BRANDS } from '../../avatar';
import type { BrandMarkPickerProps } from './BrandMarkPicker.type';

const BrandMarkPicker = (props: BrandMarkPickerProps) => {
  const { value, onPick, label, disabled = false } = props;
  return (
    <Flex gap="sm" align="center" wrap role="group" aria-label={label}>
      {AVATAR_BRANDS.map((app) => (
        <Pressable
          key={app}
          aria-label={`Use the ${BRAND_FAMILY[app].name} mark`}
          aria-pressed={value === app}
          disabled={disabled}
          onClick={() => onPick(app)}
        >
          <BrandMark app={app} size="lg" variant="app-icon" title={BRAND_FAMILY[app].name} />
        </Pressable>
      ))}
    </Flex>
  );
};

export { BrandMarkPicker };
