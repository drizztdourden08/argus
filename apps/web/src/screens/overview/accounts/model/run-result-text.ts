/* @layer renderer-app @kind logic */
import type { RunResult } from '../../../../data/types';

const summaryOf = (result: RunResult): string =>
  result.writes === undefined ? '' : `${result.writes} files written`;

const runResultText = (result: RunResult | string | null | undefined): string => {
  if (result === null || result === undefined) return '';
  if (typeof result === 'string') return result;
  return result.error ?? summaryOf(result);
};

export { runResultText };
