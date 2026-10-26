/* @layer renderer-app @kind logic */
import { toast } from '@drizztdourden08/tessera';
import { useCallback, useState } from 'react';
import { imageCaption, imageProblem, writeErrorText } from '../../../../../data/api';
import { attachProof } from '../../../../../data/posts';
import type { ProofUpload } from '../tests.type';

const useProofUpload = (boardKey: string): ProofUpload => {
  const [uploading, setUploading] = useState(false);
  const attach = useCallback(async (testId: string, file: File, caption: string): Promise<string | null> => {
    const problem = imageProblem(file);
    if (problem) return problem;
    setUploading(true);
    try {
      await attachProof(boardKey, testId, file, imageCaption(caption, file.name.replace(/\.[^.]+$/, '')));
      toast({ variant: 'success', message: `Screenshot attached to ${testId}.` });
      return null;
    } catch (error) {
      const failure = `Screenshot not attached: ${writeErrorText(error)}`;
      toast({ variant: 'danger', message: failure });
      return failure;
    } finally {
      setUploading(false);
    }
  }, [boardKey]);
  return { uploading, attach };
};

export { useProofUpload };
