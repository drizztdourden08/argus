/* @layer renderer-app @kind logic */
import type { TestPingResult } from '../../../../data/types';
import { orText } from '../../format';

const testSentence = (result: TestPingResult | null): string => {
  if (result?.toast === 'shown') return 'Sent: the toast should be on your screen now.';
  return `No toast: ${orText(result?.why, orText(result?.toast, 'unknown reason'))}. It is recorded below.`;
};

export { testSentence };
