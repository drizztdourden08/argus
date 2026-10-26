/* @layer renderer-app @kind component */
import type { DropZoneStatus } from '@drizztdourden08/tessera';
import { useState } from 'react';
import { ScreenshotAttach } from '../../../screenshots';
import { useProofUpload } from '../behavior/useProofUpload';
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
  return <ScreenshotAttach label={testId} caption={caption} onCaptionChange={setCaption} uploading={uploading} status={status} onDrop={onDrop} />;
};

export { ProofUploadForm };
