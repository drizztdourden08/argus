/* @layer renderer-app @kind component */
import { DropZone, Flex, TextInput } from '@drizztdourden08/tessera';
import { IMAGE_TYPES } from '../../../data/api';
import type { ScreenshotAttachProps } from './screenshots.type';

const ScreenshotAttach = (props: ScreenshotAttachProps) => {
  const { label, caption, onCaptionChange, uploading, status, onDrop } = props;
  return (
    <Flex direction="column" gap="xs">
      <TextInput value={caption} placeholder="Caption (optional)" aria-label={`Caption for ${label}`} onChange={(event) => onCaptionChange(event.target.value)} />
      <DropZone accept={IMAGE_TYPES} label="Drop, paste or choose a screenshot" hint="png, jpeg, gif or webp, up to 10 MB" status={status} disabled={uploading} onDrop={onDrop} />
    </Flex>
  );
};

export { ScreenshotAttach };
