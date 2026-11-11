/* @layer renderer-app @kind logic */
import { PORTFOLIO_SCOPE, useCollection, useDoc } from '../../../../data/react';
import type { AccountDoc, AccountsDrift } from '../../../../data/types';
import { ACCOUNTS_COLL, ACCOUNTS_STATE_COLL, DRIFT_DOC } from '../accounts.constants';
import type { AccountsFeed } from '../accounts.type';

const useAccountsFeed = (): AccountsFeed => {
  const accounts = useCollection<AccountDoc>(PORTFOLIO_SCOPE, ACCOUNTS_COLL);
  const drift = useDoc<AccountsDrift>(PORTFOLIO_SCOPE, ACCOUNTS_STATE_COLL, DRIFT_DOC);
  return { ready: accounts.ready, accounts: accounts.docs, drift: drift.doc };
};

export { useAccountsFeed };
