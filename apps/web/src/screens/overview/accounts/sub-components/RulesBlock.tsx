/* @layer renderer-app @kind component */
import { Inline, Stack, Status } from '@drizztdourden08/tessera';
import { CURRENT_LABEL, CURRENT_TIP } from '../accounts.constants';
import type { RulesBlockProps } from '../cards.type';
import { AccountName } from './AccountName';
import { RuleFields } from './RuleFields';

const RulesBlock = (props: RulesBlockProps) => {
  const { account, rules } = props;
  return (
    <Stack gap="sm">
      <Inline gap="sm" align="center" wrap>
        <AccountName account={account} />
        {account.active === true && <Status tone="info" size="sm" dot title={CURRENT_TIP}>{CURRENT_LABEL}</Status>}
      </Inline>
      <RuleFields accountKey={account.key ?? account.id} rules={rules} />
    </Stack>
  );
};

export { RulesBlock };
