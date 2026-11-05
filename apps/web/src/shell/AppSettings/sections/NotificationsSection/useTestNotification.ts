/* @layer renderer-app @kind logic */
import { useCallback, useState } from 'react';
import type { TestPingResult } from '../../../../data/types';
import { failureLine, IDLE_LINE, resultLine } from './test-line';

interface TestNotification {
  line: string;
  busy: boolean;
  send: () => void;
}

const useTestNotification = (run: () => Promise<TestPingResult>): TestNotification => {
  const [line, setLine] = useState(IDLE_LINE);
  const [busy, setBusy] = useState(false);
  const send = useCallback(() => {
    setBusy(true);
    run()
      .then((result) => setLine(resultLine(result)))
      .catch((error: unknown) => setLine(failureLine(error)))
      .finally(() => setBusy(false));
  }, [run]);
  return { line, busy, send };
};

export { useTestNotification };
export type { TestNotification };
