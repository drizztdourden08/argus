/* @layer renderer-app @kind component */
import { Inline, Stack, Text } from '@drizztdourden08/tessera';
import { HANDOVER_STEP, STEP_MARK, STEP_TONE } from '../switch.constants';
import type { SwitchStepsProps } from '../switch.type';
import { HandoverWait } from './HandoverWait';

const SwitchSteps = (props: SwitchStepsProps) => {
  const steps = props.feed.doc?.steps ?? [];
  return (
    <Stack gap="xs">
      {steps.map((step) => {
        const status = step.status ?? 'pending';
        return (
          <Inline key={step.id} gap="sm" align="center" wrap>
            <Text tone={STEP_TONE[status]}>{STEP_MARK[status]}</Text>
            <Text weight={status === 'running' ? 600 : undefined}>{step.label ?? step.id}</Text>
            {step.detail && <Text variant="caption" tone="muted">{step.detail}</Text>}
            {step.id === HANDOVER_STEP && status === 'running' && <HandoverWait {...props} />}
          </Inline>
        );
      })}
    </Stack>
  );
};

export { SwitchSteps };
