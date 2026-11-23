/* @layer renderer-app @kind component */
import { ProgressBar, Stack, Text } from '@drizztdourden08/tessera';
import { METER_TONE } from '../accounts.constants';
import type { LimitMeterProps } from '../cards.type';
import { limitLabel, resetText } from '../model';

const toneOf = (severity: string | undefined) => METER_TONE[severity === 'critical' || severity === 'warning' ? severity : 'normal'];

const LimitMeter = (props: LimitMeterProps) => {
  const { limit, now } = props;
  const label = limitLabel(limit);
  const reset = resetText(limit.resetsAt, now);
  return (
    <Stack gap="2xs">
      <Text variant="caption" tone="muted">{label}</Text>
      <ProgressBar value={Math.round(limit.percent ?? 0)} max={100} tone={toneOf(limit.severity)} label={label} formatValue={(value) => `${value}%`} showValue />
      {reset && <Text variant="caption" tone="muted">{reset}</Text>}
    </Stack>
  );
};

export { LimitMeter };
