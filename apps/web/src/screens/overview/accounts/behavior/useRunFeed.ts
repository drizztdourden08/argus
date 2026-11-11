/* @layer renderer-app @kind logic */
import { useMemo } from 'react';
import { PORTFOLIO_SCOPE, useCollection, useDoc } from '../../../../data/react';
import type { AccountsRun } from '../../../../data/types';
import { ACCOUNTS_STATE_COLL } from '../accounts.constants';
import { RUN_DOC, SYNC_STEPS_COLL } from '../compare.constants';
import type { RunFeed, SyncStep } from '../compare.type';
import { orderedSteps } from '../model';

const useRunFeed = (): RunFeed => {
  const run = useDoc<AccountsRun>(PORTFOLIO_SCOPE, ACCOUNTS_STATE_COLL, RUN_DOC);
  const steps = useCollection<SyncStep>(PORTFOLIO_SCOPE, SYNC_STEPS_COLL);
  const ordered = useMemo(() => orderedSteps(steps.docs), [steps.docs]);
  return { run: run.doc, steps: ordered };
};

export { useRunFeed };
