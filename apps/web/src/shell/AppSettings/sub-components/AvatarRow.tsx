/* @layer renderer-app @kind component */
import { DropZone, Flex, SegmentedControl } from '@drizztdourden08/tessera';
import { AvatarReset } from '../../AvatarReset';
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
        <DropZone variant="inline" size="sm" accept={UPLOAD_TYPES} label="Upload a PNG or SVG" disabled={choice.busy} onDrop={(files) => void choice.upload(files)} onReject={choice.reject} />
      )}
      <AvatarReset disabled={choice.busy || choice.avatar === null} problem={choice.problem} onReset={() => void choice.reset()} />
    </Flex>
  );
};

export { AvatarRow };
