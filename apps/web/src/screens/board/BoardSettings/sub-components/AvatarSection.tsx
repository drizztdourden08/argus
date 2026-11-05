/* @layer renderer-app @kind component */
import { DropZone, Flex, Text } from '@drizztdourden08/tessera';
import { AVATAR_BRANDS, AVATAR_TYPES, BRAND_PREFIX } from '../../../../avatar';
import { AvatarReset } from '../../../../shell/AvatarReset';
import { BrandMarkPicker } from '../../../../shell/BrandMarkPicker';
import { useAvatarChoice } from '../behavior/useAvatarChoice';
import { AVATAR_NOTE } from '../BoardSettings.constants';
import type { SectionProps } from '../BoardSettings.type';

const AvatarSection = (props: SectionProps) => {
  const choice = useAvatarChoice(props.boardKey);
  return (
    <Flex direction="column" gap="md">
      <Text variant="label">Lead avatar</Text>
      <Text variant="caption" tone="muted">{AVATAR_NOTE}</Text>
      <BrandMarkPicker
        label="Avatar brand marks"
        value={AVATAR_BRANDS.find((app) => choice.setting === `${BRAND_PREFIX}${app}`) ?? null}
        disabled={choice.busy}
        onPick={(app) => void choice.pickBrand(app)}
      />
      <DropZone variant="inline" size="sm" accept={AVATAR_TYPES} label="Upload an image" disabled={choice.busy} onDrop={(files) => void choice.upload(files)} />
      <AvatarReset disabled={choice.busy || choice.setting === null} problem={choice.problem} onReset={() => void choice.reset()} />
    </Flex>
  );
};

export { AvatarSection };
