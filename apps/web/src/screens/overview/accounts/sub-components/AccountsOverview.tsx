/* @layer renderer-app @kind component */
import { AsideLayout, Stack } from '@drizztdourden08/tessera';
import { useState } from 'react';
import { useAccountsSettings } from '../../../../data/accounts';
import { ASIDE_LABEL } from '../accounts.constants';
import { useAccountActions } from '../behavior/useAccountActions';
import { useCompareFeed } from '../behavior/useCompareFeed';
import { useRunFeed } from '../behavior/useRunFeed';
import type { AccountsOverviewProps } from '../cards.type';
import { isActive, useSwitchFeed, useSwitchTarget } from '../switch';
import { AccountCards } from './AccountCards';
import { ActivityCard } from './ActivityCard';
import { RulesCard } from './RulesCard';
import { SyncCard } from './SyncCard';

const AccountsOverview = (props: AccountsOverviewProps) => {
  const { accounts, drift, onReview } = props;
  const [asideShown, setAsideShown] = useState(false);
  const rules = useAccountsSettings();
  const target = useSwitchTarget();
  const doc = useSwitchFeed();
  const actions = useAccountActions();
  const { run, steps } = useRunFeed();
  const compare = useCompareFeed(`${drift?.scannedAt ?? ''}|${run?.finishedAt ?? ''}`);
  const activity = (
    <ActivityCard doc={doc} accounts={accounts} confirmKey={target.key} drifted={(drift?.total ?? 0) > 0} run={run} steps={steps} onCloseConfirm={target.close} />
  );
  const main = (
    <Stack gap="md">
      <AccountCards accounts={accounts} rules={rules} compare={compare} confirmKey={target.key} switching={isActive(doc)} onSwitch={target.open} />
      {!asideShown && activity}
      <RulesCard accounts={accounts} rules={rules} />
      <SyncCard drift={drift} accounts={accounts} run={run} actions={actions} onReview={onReview} />
    </Stack>
  );
  return <AsideLayout side="end" asideWidth="md" asideLabel={ASIDE_LABEL} onAsideChange={setAsideShown} main={main} aside={activity} />;
};

export { AccountsOverview };
