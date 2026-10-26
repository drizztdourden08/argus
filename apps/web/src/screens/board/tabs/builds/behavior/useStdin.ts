/* @layer renderer-app @kind logic */
import { toast } from '@drizztdourden08/tessera';
import { useCallback } from 'react';
import { errorMessage } from '../../../../../data/api';
import { sendStdin } from '../../../../../data/runs';

const useStdin = (boardKey: string, runId: string): ((text: string) => void) =>
  useCallback((text: string) => {
    sendStdin(boardKey, runId, text).catch((error: unknown) => {
      toast({ variant: 'danger', message: `Could not send to ${runId}: ${errorMessage(error)}` });
    });
  }, [boardKey, runId]);

export { useStdin };
