/* @layer renderer-app @kind component */
import { Flex, ProgressBar, SettingsSection, Stack, Text } from '@drizztdourden08/tessera';
import { spendLook } from '../behavior/spend-look';
import { NO_LIMIT, SPEND_NOTE } from '../OverlordPanel.constants';
import type { SpendSectionProps } from '../OverlordPanel.type';

const SpendSection = (props: SpendSectionProps) => {
  const { spend } = props;
  const look = spendLook(spend);
  const control = (
    <Stack gap="xs">
      <Flex gap="sm" align="baseline" wrap>
        <Text variant="title">{`${spend.spentUsd.toFixed(2)} USD`}</Text>
        <Text variant="caption" tone="muted">{look ? `of ${spend.budgetUsd?.toFixed(2)}` : NO_LIMIT}</Text>
      </Flex>
      {look && <ProgressBar value={look.ratio} max={1} tone={look.tone} label="Spend against the monthly budget" />}
      {look && spend.leftUsd !== null && <Text variant="caption" tone="muted">{`${spend.leftUsd.toFixed(2)} USD left.`}</Text>}
    </Stack>
  );
  const row = { id: 'spend', title: spend.month, description: SPEND_NOTE, input: { kind: 'custom' as const, control } };
  return <SettingsSection title="Spend this month" rows={[row]} />;
};

export { SpendSection };
