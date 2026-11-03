/* @layer renderer-app @kind component */
import { Disclosure, Flex, Status, Text } from '@drizztdourden08/tessera';
import { useState } from 'react';
import { CHECK_STATUSES, StepLogBody, stepStatus } from '../../../../screens/board/tabs/changes';
import type { StepLogProps } from './checks.type';

const StepLog = (props: StepLogProps) => {
  const { boardKey, run, index, first } = props;
  const step = run.steps[index];
  const [open, setOpen] = useState(first);
  if (!step) return null;
  const summary = (
    <Flex gap="sm" align="center" wrap>
      <Status map={CHECK_STATUSES} value={stepStatus(step)} dot size="sm" />
      <Text>{step.name}</Text>
      <Text variant="caption" tone="muted" mono>{step.command}</Text>
    </Flex>
  );
  return (
    <Disclosure open={open} onOpenChange={setOpen} summary={summary} size="sm">
      {open && <StepLogBody key={`${run.id}-${index}`} boardKey={boardKey} target={{ runId: run.id, index, name: step.name, command: step.command }} />}
    </Disclosure>
  );
};

export { StepLog };
