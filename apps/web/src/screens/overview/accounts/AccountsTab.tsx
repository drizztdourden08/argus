/* @layer renderer-app @kind component */
import { EmptyState, Stack } from '@drizztdourden08/tessera';
import { NONE_FOUND } from './accounts.constants';
import { useAccountActions } from './behavior/useAccountActions';
import { useAccountsFeed } from './behavior/useAccountsFeed';
import { useAccountView } from './behavior/useAccountView';
import { AccountCard } from './sub-components/AccountCard';
import { AccountsSummary } from './sub-components/AccountsSummary';
import { ApplyingView } from './sub-components/ApplyingView';
import { CompareView } from './sub-components/CompareView';
import { SyncRow } from './sub-components/SyncRow';

const AccountsTab = () => {
  const { ready, accounts, drift } = useAccountsFeed();
  const { view, show } = useAccountView();
  const actions = useAccountActions(() => show('applying'));
  if (view === 'compare') return <CompareView actions={actions} onBack={() => show('overview')} />;
  if (view === 'applying') return <ApplyingView actions={actions} onBack={() => show('overview')} />;
  if (accounts.length === 0) return ready ? <EmptyState message={NONE_FOUND} /> : null;
  const total = drift?.total ?? 0;
  return (
    <Stack gap="md">
      <AccountsSummary accounts={accounts} drift={total} />
      {accounts.map((account) => <AccountCard key={account.id} account={account} actions={actions} />)}
      <SyncRow drift={total} actions={actions} onCompare={() => show('compare')} />
    </Stack>
  );
};

export { AccountsTab };
