/* @layer renderer-app @kind component */
import { DropZone, Flex, Text, TextInput } from '@drizztdourden08/tessera';
import { IMAGE_TYPES } from '../../../data/api';
import type { ScreenshotAttachProps } from './screenshots.type';

const ScreenshotAttach = (props: ScreenshotAttachProps) => {
  const { label, caption, onCaptionChange, uploading, status, onDrop } = props;
  return (
    <Flex direction="column" gap="xs">
      <Flex gap="sm" align="center" wrap>
        <Text.B>Screenshot</Text.B>
        <DropZone variant="inline" size="sm" accept={IMAGE_TYPES} label="Choose an image" disabled={uploading} onDrop={onDrop} />
        <TextInput grow size="sm" value={caption} maxLength={120} placeholder="Caption (optional)" aria-label={`Caption for ${label}`} onChange={(event) => onCaptionChange(event.target.value)} />
      </Flex>
      <Text variant="caption" tone="muted">or paste an image (Ctrl+V) while this panel is open</Text>
      <Text variant="caption" tone={status?.tone === 'error' ? 'danger' : 'success'} role="status">{status?.message}</Text>
    </Flex>
  );
};

export { ScreenshotAttach };
