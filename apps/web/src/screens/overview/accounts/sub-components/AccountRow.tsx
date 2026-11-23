/* @layer renderer-app @kind component */
import { Box, Button, Icon, IconButton, Inline, Text } from '@drizztdourden08/tessera';
import { useState } from 'react';
import { RULES_LABEL, SESSION_KIND, SWITCH_LABEL, WEEKLY_KIND } from '../accounts.constants';
import type { AccountRowProps } from '../accounts.type';
import { rowState, usageOf } from '../model';
import { SwitchConfirm } from '../switch';
import { AccountName } from './AccountName';
import { ReceiveCell } from './ReceiveCell';
import { RuleChips } from './RuleChips';
import { UsageCell } from './UsageCell';

const AccountRow = (props: AccountRowProps) => {
  const { account, rules, now, confirming, switching, onSwitch, onCloseConfirm } = props;
  const [rulesOpen, setRulesOpen] = useState(false);
  const { receive, full, until, blocked, tip } = rowState(account, rules, now, switching);
  return (
    <Box className="accounts-strip__row" data-active={account.active === true ? 'true' : undefined}>
      <AccountName account={account} />
      <UsageCell limit={usageOf(account, SESSION_KIND)} label="Session" now={now} />
      <UsageCell limit={usageOf(account, WEEKLY_KIND)} label="Weekly" now={now} />
      <ReceiveCell receive={receive} full={full} until={until} />
      <Inline gap="xs" align="center" className="accounts-strip__cell accounts-strip__actions">
        <IconButton size="xs" variant="ghost" label={RULES_LABEL} active={rulesOpen} aria-expanded={rulesOpen} onClick={() => setRulesOpen(!rulesOpen)}>
          <Icon name="sliders-horizontal" />
        </IconButton>
        <Button size="xs" variant="secondary" disabled={blocked || confirming} title={tip} onClick={() => onSwitch(account.key ?? account.id)}>{SWITCH_LABEL}</Button>
      </Inline>
      {account.error && <Text variant="caption" tone="warning" className="accounts-strip__wide">{account.error}</Text>}
      {rulesOpen && <Box className="accounts-strip__wide"><RuleChips account={account} rules={rules} /></Box>}
      {confirming && !blocked && <Box className="accounts-strip__wide"><SwitchConfirm target={account} onClose={onCloseConfirm} /></Box>}
    </Box>
  );
};

export { AccountRow };
