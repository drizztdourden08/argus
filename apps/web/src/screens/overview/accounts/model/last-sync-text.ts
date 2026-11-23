/* @layer renderer-app @kind logic */
import type { AccountsRun } from '../../../../data/types';
import { FAILED_TEXT, MATCH_TEXT, NEVER_SYNCED } from '../compare.constants';
import { clockText } from './clock-text';
import { runResultText } from './run-result-text';

const lastSyncText = (run: AccountsRun | null): string => {
  if (run?.phase !== 'done' && run?.phase !== 'failed') return NEVER_SYNCED;
  const at = clockText(run.finishedAt);
  const when = at ? ` at ${at}` : '';
  const result = runResultText(run.result);
  if (run.phase === 'failed') return `Last sync failed${when}: ${result || FAILED_TEXT}`;
  return `Last sync${when}: ${result || MATCH_TEXT}.`;
};

export { lastSyncText };
