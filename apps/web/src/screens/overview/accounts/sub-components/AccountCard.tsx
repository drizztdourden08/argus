/* @layer renderer-app @kind component */
import { Card, Stack, Status, Text } from '@drizztdourden08/tessera';
import { CURRENT_LABEL, CURRENT_TIP } from '../accounts.constants';
import type { AccountCardProps } from '../cards.type';
import { differsText, rowState } from '../model';
import { AccountFacts } from './AccountFacts';
import { AccountName } from './AccountName';
import { LimitMeters } from './LimitMeters';
import { ReceiveLine } from './ReceiveLine';
import { SwitchAction } from './SwitchAction';

const AccountCard = (props: AccountCardProps) => {
  const { account, current, rules, compare, now, confirming, switching, onSwitch } = props;
  const active = account.active === true;
  const state = rowState(account, rules, now, switching);
  const differs = active ? null : differsText(compare, account.key ?? account.id);
  return (
    <Card
      title={<AccountName account={account} />}
      actions={active ? <Status tone="info" size="sm" dot title={CURRENT_TIP}>{CURRENT_LABEL}</Status> : undefined}
      edge={active ? 'left' : undefined}
      edgeTone={active ? 'info' : undefined}
    >
      <Stack gap="md">
        {active && <Text variant="caption" tone="muted">{CURRENT_TIP}</Text>}
        <ReceiveLine state={state} />
        <LimitMeters account={account} now={now} />
        <AccountFacts account={account} differs={differs} />
        {account.error && <Text variant="caption" tone="warning">{account.error}</Text>}
        {!active && <SwitchAction account={account} current={current} state={state} switching={switching} confirming={confirming} onSwitch={onSwitch} />}
      </Stack>
    </Card>
  );
};

export { AccountCard };
