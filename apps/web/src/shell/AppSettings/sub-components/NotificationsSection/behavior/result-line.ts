/* @layer renderer-app @kind logic */
import type { TestPingResult } from '../../../../../data/types';

const resultLine = (result: TestPingResult): string => {
  const shown = result.toast === 'shown';
  const detail = result.why ?? result.toast ?? 'recorded';
  return shown ? 'Sent, the toast showed.' : `Sent, no toast: ${detail}.`;
};

export { resultLine };
