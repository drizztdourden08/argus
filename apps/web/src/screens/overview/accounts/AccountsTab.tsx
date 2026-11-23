/* @layer renderer-app @kind component */
import { EmptyState } from '@drizztdourden08/tessera';
import { useState } from 'react';
import { NONE_FOUND } from './accounts.constants';
import type { AccountsView } from './accounts.type';
import { useAccountsFeed } from './behavior/useAccountsFeed';
import { ReviewView } from './review';
import { AccountsOverview } from './sub-components/AccountsOverview';

const AccountsTab = () => {
  const { ready, accounts, drift } = useAccountsFeed();
  const [view, show] = useState<AccountsView>('overview');
  if (view === 'review') return <ReviewView onBack={() => show('overview')} onApplied={() => show('overview')} />;
  if (accounts.length === 0) return ready ? <EmptyState message={NONE_FOUND} /> : null;
  return <AccountsOverview accounts={accounts} drift={drift} onReview={() => show('review')} />;
};

export { AccountsTab };
