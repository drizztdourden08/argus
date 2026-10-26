/* @layer renderer-app @kind logic */
import { toast } from '@drizztdourden08/tessera';
import type { DropZoneStatus } from '@drizztdourden08/tessera';
import { useCallback, useEffect, useState } from 'react';
import { errorMessage, imageCaption, imageProblem } from '../../../../../data/api';
import { attachToRun, listRunAttachments } from '../../../../../data/runs';
import type { RunAttachment } from '../../../../../data/runs';
import type { RunAttachments } from '../builds.type';

const useRunAttachments = (boardKey: string, runId: string): RunAttachments => {
  const [items, setItems] = useState<RunAttachment[]>([]);
  const [uploading, setUploading] = useState(false);
  const [status, setStatus] = useState<DropZoneStatus | undefined>(undefined);
  useEffect(() => {
    let cancelled = false;
    listRunAttachments(boardKey, runId).then((found) => {
      if (!cancelled) setItems(found);
    }, () => undefined);
    return () => {
      cancelled = true;
    };
  }, [boardKey, runId]);
  const attach = useCallback(async (file: File, caption: string) => {
    const problem = imageProblem(file);
    if (problem) {
      setStatus({ tone: 'error', message: problem });
      return false;
    }
    setUploading(true);
    try {
      const { attachment } = await attachToRun(boardKey, runId, file, imageCaption(caption, file.name.replace(/\.[^.]+$/, '')));
      setItems((now) => [...now, attachment]);
      setStatus({ tone: 'success', message: 'Attached.' });
      toast({ variant: 'success', message: `Screenshot attached to ${runId}.` });
      return true;
    } catch (error) {
      setStatus({ tone: 'error', message: `Not attached: ${errorMessage(error)}` });
      return false;
    } finally {
      setUploading(false);
    }
  }, [boardKey, runId]);
  return { items, uploading, status, attach };
};

export { useRunAttachments };
