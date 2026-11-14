/* @layer renderer-app @kind logic */
import { useState } from 'react';
import { errorMessage } from '../../api';
import { useRemoteRead } from '../../remote';
import type { KeyStatus, OverlordKeyApi } from '../overlord.type';
import { readKey } from '../read-key';
import { removeKey } from '../remove-key';
import { saveKey } from '../save-key';

const useOverlordKey = (): OverlordKeyApi => {
  const read = useRemoteRead(readKey);
  const [busy, setBusy] = useState(false);
  const [problem, setProblem] = useState<string | null>(null);

  const run = async (action: () => Promise<KeyStatus>): Promise<boolean> => {
    setBusy(true);
    setProblem(null);
    try {
      await action();
      read.reload();
      return true;
    } catch (failure) {
      setProblem(errorMessage(failure));
      return false;
    } finally {
      setBusy(false);
    }
  };

  return {
    status: read.data,
    busy,
    problem: problem ?? (read.error || null),
    save: (key) => run(() => saveKey(key)),
    remove: () => run(removeKey),
  };
};

export { useOverlordKey };
