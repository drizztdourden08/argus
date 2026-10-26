/* @layer renderer-app @kind logic */
import { toast } from '@drizztdourden08/tessera';
import { useCallback } from 'react';
import { errorMessage } from '../../../../../data/api';
import { sendStdin } from '../../../../../data/runs';

const useStdin = (boardKey: string, runId: string): ((text: string) => Promise<boolean>) =>
  useCallback(
    async (text: string): Promise<boolean> => {
      try {
        await sendStdin(boardKey, runId, text);
        return true;
      } catch (error) {
        toast({ variant: 'danger', message: `Could not send to ${runId}: ${errorMessage(error)}` });
        return false;
      }
    },
    [boardKey, runId],
  );

export { useStdin };
