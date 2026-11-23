/* @layer renderer-app @kind component */
import { Flex, ProgressBar, Stack, Text } from '@drizztdourden08/tessera';
import { Panel } from '../../../../shell';
import { spendLook } from '../model';
import { NO_LIMIT, SPEND_NOTE } from '../overlord.constants';
import type { SpendCardProps } from '../overlord.type';

const SpendCard = (props: SpendCardProps) => {
  const { spend } = props;
  const look = spendLook(spend);
  return (
    <Panel title="Spend this month" note={spend.month}>
      <Stack gap="sm">
        <Flex gap="sm" align="baseline" wrap>
          <Text variant="title">{`${spend.spentUsd.toFixed(2)} USD`}</Text>
          <Text variant="caption" tone="muted">{look ? `of ${spend.budgetUsd?.toFixed(2)}` : NO_LIMIT}</Text>
        </Flex>
        {look && <ProgressBar value={look.ratio} max={1} tone={look.tone} label="Spend against the monthly budget" />}
        {look && spend.leftUsd !== null && <Text variant="caption" tone="muted">{`${spend.leftUsd.toFixed(2)} USD left.`}</Text>}
        <Text variant="caption" tone="muted">{SPEND_NOTE}</Text>
      </Stack>
    </Panel>
  );
};

export { SpendCard };
