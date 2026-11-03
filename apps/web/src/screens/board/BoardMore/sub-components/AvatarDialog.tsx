/* @layer renderer-app @kind component */
import { Button, DialogShell, DropZone, Flex, Pressable, Text } from '@drizztdourden08/tessera';
import { BRAND_FAMILY, BrandMark } from '@drizztdourden08/tessera/brand';
import { AVATAR_BRANDS, AVATAR_TYPES, BRAND_PREFIX } from '../../../../avatar';
import { useAvatarChoice } from '../behavior/useAvatarChoice';
import { AVATAR_NOTE } from '../BoardMore.constants';
import type { AvatarDialogProps } from '../BoardMore.type';

const AvatarDialog = (props: AvatarDialogProps) => {
  const { boardKey, onClose } = props;
  const choice = useAvatarChoice(boardKey, onClose);
  return (
    <DialogShell
      open
      size="md"
      title="Lead avatar"
      onClose={onClose}
      actions={<Button size="sm" variant="ghost" onClick={onClose}>Close</Button>}
    >
      <Flex direction="column" gap="md">
        <Text variant="caption" tone="muted">{AVATAR_NOTE}</Text>
        <Flex gap="sm" align="center" wrap role="group" aria-label="Brand marks">
          {AVATAR_BRANDS.map((app) => (
            <Pressable
              key={app}
              aria-label={`Use the ${BRAND_FAMILY[app].name} mark`}
              aria-pressed={choice.setting === `${BRAND_PREFIX}${app}`}
              disabled={choice.busy}
              onClick={() => void choice.pickBrand(app)}
            >
              <BrandMark app={app} size="lg" variant="app-icon" title={BRAND_FAMILY[app].name} />
            </Pressable>
          ))}
        </Flex>
        <DropZone variant="inline" size="sm" accept={AVATAR_TYPES} label="Upload an image" disabled={choice.busy} onDrop={(files) => void choice.upload(files)} />
        <Flex gap="sm" align="center">
          <Button size="sm" variant="secondary" disabled={choice.busy || choice.setting === null} onClick={() => void choice.reset()}>Use the default</Button>
          {choice.problem && <Text variant="caption" tone="danger" role="alert">{choice.problem}</Text>}
        </Flex>
      </Flex>
    </DialogShell>
  );
};

export { AvatarDialog };
