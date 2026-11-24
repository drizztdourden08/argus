/* @layer renderer-app @kind component */
import { Stack, Tabs } from '@drizztdourden08/tessera';
import { useState } from 'react';
import { useAccountsSettings } from '../../../../data/accounts';
import { useAccountActions } from '../behavior/useAccountActions';
import { useCompareFeed } from '../behavior/useCompareFeed';
import { useNextActions } from '../behavior/useNextActions';
import { useNextFeed } from '../behavior/useNextFeed';
import { useRunFeed } from '../behavior/useRunFeed';
import type { AccountsOverviewProps } from '../cards.type';
import { findAccount } from '../model';
import { ACCOUNT_TABS, TAB_SWITCH, TAB_SYNC } from '../next.constants';
import { isActive, useSwitchFeed, useSwitchTarget } from '../switch';
import { AccountPane } from './AccountPane';
import { SwitchTab } from './SwitchTab';
import { SyncCard } from './SyncCard';

const AccountsOverview = (props: AccountsOverviewProps) => {
  const { accounts, drift, onReview } = props;
  const [tab, setTab] = useState(TAB_SWITCH);
  const [selected, setSelected] = useState<string | null>(null);
  const rules = useAccountsSettings();
  const target = useSwitchTarget();
  const doc = useSwitchFeed();
  const actions = useAccountActions();
  const { run, steps } = useRunFeed();
  const feed = useNextFeed();
  const next = useNextActions(feed.reload);
  const compare = useCompareFeed(`${drift?.scannedAt ?? ''}|${run?.finishedAt ?? ''}`);
  const open = findAccount(accounts, selected ?? target.key);
  const close = () => {
    setSelected(null);
    target.close();
  };
  return (
    <Stack gap="sm">
      <Tabs tabs={ACCOUNT_TABS} activeTab={tab} selectedTone="neutral" onTabChange={setTab} />
      {tab === TAB_SWITCH && <SwitchTab accounts={accounts} status={feed.status} actions={next} doc={doc} onOpen={setSelected} />}
      {tab === TAB_SYNC && <SyncCard drift={drift} accounts={accounts} run={run} steps={steps} actions={actions} onReview={onReview} />}
      {open && (
        <AccountPane
          account={open}
          accounts={accounts}
          rules={rules}
          compare={compare}
          confirming={findAccount(accounts, target.key)?.id === open.id}
          switching={isActive(doc)}
          onSwitch={target.open}
          onCloseConfirm={target.close}
          onClose={close}
        />
      )}
    </Stack>
  );
};

export { AccountsOverview };
