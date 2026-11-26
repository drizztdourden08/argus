/* @layer renderer-app @kind component */
import { Inline, Tag, Text } from '@drizztdourden08/tessera';
import type { AccountNameProps } from '../cards.type';

const AccountName = (props: AccountNameProps) => {
  const { account } = props;
  const label = account.label ?? account.key ?? 'Account';
  return (
    <Inline gap="xs" align="center">
      {account.color?.startsWith('#') && <Tag variant="custom" color={account.color as `#${string}`} pill>{label.slice(0, 1).toUpperCase()}</Tag>}
      <Text weight={600} clamp={1}>{label}</Text>
      {account.plan && <Text variant="caption" tone="muted">{account.plan}</Text>}
    </Inline>
  );
};

export { AccountName };
