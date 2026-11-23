/* @layer renderer-app @kind component */
import { Box, Button, Icon, IconButton, Inline, Stack, Status, Text } from '@drizztdourden08/tessera';
import { useState } from 'react';
import { AT_LIMIT, limitTip, limitUntil, RECEIVE_STATUSES, RULES_LABEL, SESSION_KIND, SWITCH_LABEL, WEEKLY_KIND } from '../accounts.constants';
import type { AccountRowProps } from '../accounts.type';
import { canReceive, resetClock, usageOf, weeklyLimit } from '../model';
import { SwitchConfirm } from '../switch';
import { AccountName } from './AccountName';
import { RuleChips } from './RuleChips';
import { UsageCell } from './UsageCell';

const AccountRow = (props: AccountRowProps) => {
  const { account, rules, now, confirming, switching, onSwitch, onCloseConfirm } = props;
  const [rulesOpen, setRulesOpen] = useState(false);
  const receive = canReceive(account, rules);
  const full = weeklyLimit(account);
  const until = resetClock(full?.resetsAt, now);
  const blocked = account.active === true || switching || !receive || full !== null;
  return (
    <Box className="accounts-strip__row" data-active={account.active === true ? 'true' : undefined}>
      <AccountName account={account} />
      <UsageCell limit={usageOf(account, SESSION_KIND)} label="Session" now={now} />
      <UsageCell limit={usageOf(account, WEEKLY_KIND)} label="Weekly" now={now} />
      {full ? (
        <Stack gap="2xs" className="accounts-strip__cell">
          <Status tone="danger" size="sm" dot>{AT_LIMIT}</Status>
          {until && <Text variant="caption" tone="muted">{limitUntil(until)}</Text>}
        </Stack>
      ) : (
        <Box className="accounts-strip__cell"><Status map={RECEIVE_STATUSES} value={receive ? 'open' : 'locked'} size="sm" dot /></Box>
      )}
      <Inline gap="xs" align="center" className="accounts-strip__cell accounts-strip__actions">
        <IconButton size="xs" variant="ghost" label={RULES_LABEL} active={rulesOpen} aria-expanded={rulesOpen} onClick={() => setRulesOpen(!rulesOpen)}>
          <Icon name="sliders-horizontal" />
        </IconButton>
        <Button size="xs" variant="secondary" disabled={blocked || confirming} title={full ? limitTip(until) : undefined} onClick={() => onSwitch(account.key ?? account.id)}>{SWITCH_LABEL}</Button>
      </Inline>
      {account.error && <Text variant="caption" tone="warning" className="accounts-strip__wide">{account.error}</Text>}
      {rulesOpen && <Box className="accounts-strip__wide"><RuleChips account={account} rules={rules} /></Box>}
      {confirming && !blocked && <Box className="accounts-strip__wide"><SwitchConfirm target={account} onClose={onCloseConfirm} /></Box>}
    </Box>
  );
};

export { AccountRow };
