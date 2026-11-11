/* @layer renderer-app @kind logic */
import { useState } from 'react';
import { PORTFOLIO_SCOPE, useDoc } from '../../../../data/react';
import type { AccountsRun } from '../../../../data/types';
import { ACCOUNTS_STATE_COLL } from '../accounts.constants';
import type { AccountsView } from '../accounts.type';
import { ACTIVE_PHASES, RUN_DOC } from '../compare.constants';

interface AccountViewState {
  view: AccountsView;
  show: (next: AccountsView) => void;
}

const useAccountView = (): AccountViewState => {
  const [chosen, show] = useState<AccountsView>('overview');
  const run = useDoc<AccountsRun>(PORTFOLIO_SCOPE, ACCOUNTS_STATE_COLL, RUN_DOC);
  const phase = run.doc?.phase;
  const running = phase !== undefined && ACTIVE_PHASES.has(phase);
  return { view: running ? 'applying' : chosen, show };
};

export { useAccountView };
