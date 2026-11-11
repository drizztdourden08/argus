/* @layer renderer-app @kind component */
import { Button, Callout, Card, Inline, Stack, Tag, Text } from '@drizztdourden08/tessera';
import { ACTIVE_NOTE, SEVERITY_LOOK } from '../accounts.constants';
import type { AccountCardProps } from '../accounts.type';
import { severityOf } from '../model';
import { AccountFacts } from './AccountFacts';
import { LimitMeter } from './LimitMeter';
import { RuleChips } from './RuleChips';

const AccountCard = (props: AccountCardProps) => {
  const { account, actions } = props;
  const look = SEVERITY_LOOK[severityOf(account)];
  const label = account.label ?? account.key ?? 'Account';
  const limits = (account.limits ?? []).filter((limit) => limit.isActive !== false);
  return (
    <Card edge="left" edgeTone={look.color}>
      <Stack gap="sm">
        <Inline gap="sm" align="center" wrap>
          {account.color?.startsWith('#') && <Tag variant="custom" color={account.color as `#${string}`} pill>{label.slice(0, 1).toUpperCase()}</Tag>}
          <Text size="md" weight={600}>{label}</Text>
          {account.plan && <Tag>{account.plan}</Tag>}
          {account.active && <Tag variant="urgency" color="info" dot>{ACTIVE_NOTE}</Tag>}
          <Tag variant="urgency" color={look.color}>{look.label}</Tag>
        </Inline>
        {account.error && <Callout tone="warning">{account.error}</Callout>}
        {limits.map((limit) => <LimitMeter key={`${limit.kind}-${limit.model ?? ''}`} limit={limit} />)}
        <AccountFacts account={account} />
        <RuleChips account={account} />
        <Inline gap="sm">
          <Button size="sm" disabled={account.active === true || actions.busy} onClick={() => actions.switchTo(account.key ?? account.id)}>Switch here</Button>
          <Button size="sm" variant="secondary" disabled={actions.busy} onClick={actions.sync}>Sync only</Button>
        </Inline>
      </Stack>
    </Card>
  );
};

export { AccountCard };
