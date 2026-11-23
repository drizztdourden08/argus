/* @layer renderer-app @kind logic */
import type { AccountsRun } from '../../../../data/types';
import { FAILED_TEXT, MATCH_TEXT, NEVER_SYNCED } from '../compare.constants';
import { clockText } from './clock-text';
import { runResultText } from './run-result-text';

const whenText = (run: AccountsRun): string => {
  const at = clockText(run.finishedAt);
  return at ? ` at ${at}` : '';
};

const doneText = (run: AccountsRun, drifted: boolean): string => {
  const tail = runResultText(run.result) || (drifted ? '' : MATCH_TEXT);
  return `Last sync${whenText(run)}${tail ? `: ${tail}` : ''}.`;
};

const lastSyncText = (run: AccountsRun | null, drifted = false): string => {
  if (run?.phase === 'failed') return `Last sync failed${whenText(run)}: ${runResultText(run.result) || FAILED_TEXT}`;
  return run?.phase === 'done' ? doneText(run, drifted) : NEVER_SYNCED;
};

export { lastSyncText };
