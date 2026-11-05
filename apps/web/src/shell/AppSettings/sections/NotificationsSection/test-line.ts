/* @layer renderer-app @kind logic */
import { errorMessage } from '../../../../data/api';
import type { TestPingResult } from '../../../../data/types';

const IDLE_LINE = 'Sends one ping through the same path as a real one.';

const resultLine = (result: TestPingResult): string => {
  const shown = result.toast === 'shown' || result.toast === 'ok';
  const detail = result.why ?? result.toast ?? 'recorded';
  return shown ? 'Sent, the toast showed.' : `Sent, no toast: ${detail}.`;
};

const failureLine = (error: unknown): string => `The test failed: ${errorMessage(error)}`;

export { failureLine, IDLE_LINE, resultLine };
