/* @layer renderer-app @kind component */
import { EmptyState, Stack } from '@drizztdourden08/tessera';
import { useState } from 'react';
import { useAccountsSettings } from '../../../data/accounts';
import { NONE_FOUND } from './accounts.constants';
import type { AccountsView } from './accounts.type';
import { useAccountActions } from './behavior/useAccountActions';
import { useAccountsFeed } from './behavior/useAccountsFeed';
import { ReviewView } from './review';
import { AccountsStrip } from './sub-components/AccountsStrip';
import { SyncSection } from './sub-components/SyncSection';
import { isActive, ProcessPanel, useSwitchFeed, useSwitchTarget } from './switch';

const AccountsTab = () => {
  const { ready, accounts, drift } = useAccountsFeed();
  const [view, show] = useState<AccountsView>('overview');
  const rules = useAccountsSettings();
  const target = useSwitchTarget();
  const doc = useSwitchFeed();
  const actions = useAccountActions();
  if (view === 'review') return <ReviewView onBack={() => show('overview')} onApplied={() => show('overview')} />;
  if (accounts.length === 0) return ready ? <EmptyState message={NONE_FOUND} /> : null;
  return (
    <Stack gap="md">
      <AccountsStrip accounts={accounts} rules={rules} confirmKey={target.key} switching={isActive(doc)} onSwitch={target.open} onCloseConfirm={target.close} />
      <ProcessPanel doc={doc} accounts={accounts} />
      <SyncSection drift={drift} accounts={accounts} actions={actions} onReview={() => show('review')} />
    </Stack>
  );
};

export { AccountsTab };
