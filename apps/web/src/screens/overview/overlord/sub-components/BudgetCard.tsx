/* @layer renderer-app @kind component */
import { Button, Callout, Flex, NumberInput, Stack, Text } from '@drizztdourden08/tessera';
import { useState } from 'react';
import { Panel } from '../../../../shell';
import { BUDGET_NOTE, NO_LIMIT } from '../overlord.constants';
import type { BudgetCardProps } from '../overlord.type';

const BudgetCard = (props: BudgetCardProps) => {
  const { spend, action, onSave } = props;
  const [draft, setDraft] = useState(spend.budgetUsd ?? 0);
  const current = spend.budgetUsd === null ? NO_LIMIT : `${spend.budgetUsd.toFixed(2)} USD a month`;
  return (
    <Panel title="Budget">
      <Stack gap="sm">
        <Text variant="caption" tone="muted">{`Current: ${current}`}</Text>
        <Flex gap="sm" align="center" wrap>
          <NumberInput aria-label="Monthly budget in USD" min={0} step={1} value={draft} onChange={setDraft} />
          <Text variant="caption" tone="muted">USD a month</Text>
          <Button disabled={action.busy || draft <= 0 || draft === spend.budgetUsd} onClick={() => onSave(draft)}>Save</Button>
          <Button disabled={action.busy || spend.budgetUsd === null} onClick={() => onSave(null)}>No limit</Button>
        </Flex>
        <Text variant="caption" tone="muted">{BUDGET_NOTE}</Text>
        {action.problem && <Callout tone="danger">{action.problem}</Callout>}
      </Stack>
    </Panel>
  );
};

export { BudgetCard };
