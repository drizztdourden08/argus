/* @layer renderer-app @kind component */
import { Card, Divider, Stack } from '@drizztdourden08/tessera';
import { Fragment } from 'react';
import { RULES_SUBTITLE, RULES_TITLE } from '../accounts.constants';
import type { RulesCardProps } from '../cards.type';
import { orderAccounts } from '../model';
import { RulesBlock } from './RulesBlock';

const RulesCard = (props: RulesCardProps) => {
  const { accounts, rules } = props;
  return (
    <Card title={RULES_TITLE} subtitle={RULES_SUBTITLE}>
      <Stack gap="md">
        {orderAccounts(accounts).map((account, index) => (
          <Fragment key={account.id}>
            {index > 0 && <Divider />}
            <RulesBlock account={account} rules={rules} />
          </Fragment>
        ))}
      </Stack>
    </Card>
  );
};

export { RulesCard };
