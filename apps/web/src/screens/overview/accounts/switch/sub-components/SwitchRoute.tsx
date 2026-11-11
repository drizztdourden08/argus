/* @layer renderer-app @kind component */
import { Icon, Inline, Text } from '@drizztdourden08/tessera';
import type { Account } from '../../accounts.type';
import { weeklyPercent } from '../../model';

const side = (account: Account | undefined, fallback: string): string => {
  const percent = weeklyPercent(account);
  return `${account?.label ?? fallback}${percent === null ? '' : ` · weekly ${percent}%`}`;
};

const SwitchRoute = (props: { from: Account | undefined; to: Account }) => (
  <Inline gap="sm" align="center" wrap>
    <Text>{side(props.from, 'Current')}</Text>
    <Icon name="arrow-right" size={14} aria-hidden />
    <Text weight={600}>{side(props.to, props.to.id)}</Text>
  </Inline>
);

export { SwitchRoute };
