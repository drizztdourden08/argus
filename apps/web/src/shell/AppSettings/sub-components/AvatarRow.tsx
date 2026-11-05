/* @layer renderer-app @kind component */
import { Button, DropZone, Flex, SegmentedControl, Text } from '@drizztdourden08/tessera';
import { BrandMarkPicker } from '../../BrandMarkPicker';
import { OwnerMark } from '../../OwnerMark';
import { AVATAR_MODES, PREVIEW_SIZE, UPLOAD_TYPES } from '../AppSettings.constants';
import type { ProfileChoice } from '../AppSettings.type';

const AvatarRow = (props: { choice: ProfileChoice }) => {
  const { choice } = props;
  return (
    <Flex direction="column" gap="md">
      <Flex gap="md" align="center">
        <OwnerMark size={PREVIEW_SIZE} label="Avatar preview" />
        <SegmentedControl
          selectedTone="neutral"
          size="sm"
          aria-label="Avatar source"
          value={choice.mode}
          options={AVATAR_MODES}
          onChange={(mode) => void choice.chooseMode(mode)}
        />
      </Flex>
      {choice.mode === 'brand' && (
        <BrandMarkPicker label="Avatar brand marks" value={choice.brand} disabled={choice.busy} onPick={(app) => void choice.pickBrand(app)} />
      )}
      {choice.mode === 'upload' && (
        <DropZone variant="inline" size="sm" accept={UPLOAD_TYPES} label="Upload a PNG or SVG" disabled={choice.busy} onDrop={(files) => void choice.upload(files)} />
      )}
      <Flex gap="sm" align="center">
        <Button size="sm" variant="secondary" disabled={choice.busy || choice.avatar === null} onClick={() => void choice.reset()}>Use the default</Button>
        {choice.problem && <Text variant="caption" tone="danger" role="alert">{choice.problem}</Text>}
      </Flex>
    </Flex>
  );
};

export { AvatarRow };
