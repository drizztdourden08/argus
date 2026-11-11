/* @layer renderer-app @kind component */
import { Inline, ProgressBar, Text } from '@drizztdourden08/tessera';
import { METER_TONE } from '../accounts.constants';
import type { LimitMeterProps } from '../accounts.type';
import { limitLabel, resetText } from '../model';

const LimitMeter = (props: LimitMeterProps) => {
  const { limit } = props;
  const percent = Math.round(limit.percent ?? 0);
  const tone = METER_TONE[limit.severity === 'critical' || limit.severity === 'warning' ? limit.severity : 'normal'];
  return (
    <Inline gap="sm" align="center" wrap>
      <Text size="md" weight={600}>{limitLabel(limit)}</Text>
      <ProgressBar value={percent} max={100} tone={tone} label={limitLabel(limit)} formatValue={(value) => `${value}%`} showValue />
      <Text variant="subtitle" size="md">{resetText(limit.resetsAt)}</Text>
    </Inline>
  );
};

export { LimitMeter };
