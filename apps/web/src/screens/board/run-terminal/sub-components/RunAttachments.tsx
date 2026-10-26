/* @layer renderer-app @kind component */
import { Flex, Text } from '@drizztdourden08/tessera';
import { useState } from 'react';
import { ScreenshotAttach, ScreenshotGallery } from '../../screenshots';
import { useRunAttachments } from '../behavior/useRunAttachments';
import type { RunAttachmentsProps } from '../run-terminal.type';

const RunAttachments = (props: RunAttachmentsProps) => {
  const { boardKey, runId } = props;
  const { items, uploading, status, attach } = useRunAttachments(boardKey, runId);
  const [caption, setCaption] = useState('');
  const onDrop = (files: File[]) => {
    const file = files[0];
    if (file) void attach(file, caption).then((attached) => attached && setCaption(''));
  };
  return (
    <Flex direction="column" gap="sm">
      <Text weight={600}>Screenshots</Text>
      {items.length > 0 && <ScreenshotGallery shots={items} />}
      <ScreenshotAttach label={runId} caption={caption} onCaptionChange={setCaption} uploading={uploading} status={status} onDrop={onDrop} />
    </Flex>
  );
};

export { RunAttachments };
