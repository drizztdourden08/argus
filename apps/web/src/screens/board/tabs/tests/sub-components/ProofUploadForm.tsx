/* @layer renderer-app @kind component */
import { DropZone, Flex, TextInput } from '@drizztdourden08/tessera';
import type { DropZoneStatus } from '@drizztdourden08/tessera';
import { useState } from 'react';
import { useProofUpload } from '../behavior/useProofUpload';
import { IMAGE_TYPES } from '../model';
import type { ProofUploadFormProps } from '../tests-props.type';

const ProofUploadForm = (props: ProofUploadFormProps) => {
  const { testId, boardKey } = props;
  const { uploading, attach } = useProofUpload(boardKey);
  const [caption, setCaption] = useState('');
  const [status, setStatus] = useState<DropZoneStatus | undefined>(undefined);
  const onDrop = (files: File[]) => {
    const file = files[0];
    if (!file) return;
    void attach(testId, file, caption).then((problem) => {
      setStatus(problem ? { tone: 'error', message: problem } : { tone: 'success', message: 'Attached.' });
      if (!problem) setCaption('');
    });
  };
  return (
    <Flex direction="column" gap="xs">
      <TextInput value={caption} placeholder="Caption (optional)" aria-label={`Caption for ${testId}`} onChange={(event) => setCaption(event.target.value)} />
      <DropZone accept={IMAGE_TYPES} label="Drop, paste or choose a screenshot" hint="png, jpeg, gif or webp, up to 10 MB" status={status} disabled={uploading} onDrop={onDrop} />
    </Flex>
  );
};

export { ProofUploadForm };
