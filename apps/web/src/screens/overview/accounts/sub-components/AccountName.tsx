/* @layer renderer-app @kind component */
import { Inline, Status, Tag, Text } from '@drizztdourden08/tessera';
import { ACTIVE_NOTE, ACTIVE_TIP } from '../accounts.constants';
import type { AccountNameProps } from '../accounts.type';
import { accountFacts } from '../model';

const AccountName = (props: AccountNameProps) => {
  const { account } = props;
  const label = account.label ?? account.key ?? 'Account';
  return (
    <Inline gap="xs" align="center" wrap className="accounts-strip__cell accounts-strip__name">
      {account.color?.startsWith('#') && <Tag variant="custom" color={account.color as `#${string}`} pill>{label.slice(0, 1).toUpperCase()}</Tag>}
      <Text weight={600} title={accountFacts(account)}>{label}</Text>
      {account.plan && <Text variant="caption" tone="muted">{account.plan}</Text>}
      {account.active && <Status tone="info" size="sm" dot title={ACTIVE_TIP}>{ACTIVE_NOTE}</Status>}
    </Inline>
  );
};

export { AccountName };
