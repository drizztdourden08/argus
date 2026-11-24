/* @layer renderer-app @kind component */
import { Divider, Drawer, Stack, Text } from '@drizztdourden08/tessera';
import { useNow } from '../../clock';
import { CARDS_TICK_MS, RULES_SUBTITLE, RULES_TITLE } from '../accounts.constants';
import { accountKey, currentName, differsText, rowState } from '../model';
import { PANE_LABEL } from '../next.constants';
import type { AccountPaneProps } from '../next.type';
import { SwitchConfirm } from '../switch';
import { AccountFacts } from './AccountFacts';
import { AccountName } from './AccountName';
import { LimitMeters } from './LimitMeters';
import { RuleFields } from './RuleFields';
import { SwitchAction } from './SwitchAction';

const AccountPane = (props: AccountPaneProps) => {
  const { account, accounts, rules, compare, confirming, switching, onSwitch, onCloseConfirm, onClose } = props;
  const now = useNow(CARDS_TICK_MS);
  const active = account.active === true;
  const key = accountKey(account);
  const state = rowState(account, rules, now, switching);
  const current = accounts.find((item) => item.active === true) ?? null;
  return (
    <Drawer open onClose={onClose} look="record" size="wide" label={PANE_LABEL} title={<AccountName account={account} />}>
      <Stack gap="md">
        {confirming && <SwitchConfirm target={account} from={currentName(accounts)} onClose={onCloseConfirm} />}
        <LimitMeters account={account} now={now} />
        <AccountFacts account={account} differs={active ? null : differsText(compare, key)} />
        {account.error && <Text variant="caption" tone="warning">{account.error}</Text>}
        {!active && <SwitchAction account={account} current={current} state={state} switching={switching} confirming={confirming} onSwitch={onSwitch} />}
        <Divider />
        <Text variant="overline" tone="muted">{RULES_TITLE}</Text>
        <Text variant="caption" tone="muted">{RULES_SUBTITLE}</Text>
        <RuleFields accountKey={key} rules={rules} />
      </Stack>
    </Drawer>
  );
};

export { AccountPane };
