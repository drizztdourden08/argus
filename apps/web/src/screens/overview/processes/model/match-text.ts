/* @layer renderer-app @kind logic */
import type { ProcRuleMatch } from '../../../../data/types';

const matchText = (match: ProcRuleMatch | undefined): string =>
  Object.entries(match ?? {})
    .map(([part, pattern]) => `${part}: ${pattern}`)
    .join(', ');

export { matchText };
