/* @layer renderer-app @kind component */
import { Text } from '@drizztdourden08/tessera';
import type { AccountNameProps } from '../cards.type';

const SlotName = (props: AccountNameProps) => {
  const { account } = props;
  const label = account.label ?? account.key ?? 'Account';
  return <Text variant="caption" weight={600} clamp={3} title={label}>{label}</Text>;
};

export { SlotName };
