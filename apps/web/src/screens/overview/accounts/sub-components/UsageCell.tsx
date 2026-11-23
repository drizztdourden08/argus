/* @layer renderer-app @kind component */
import { ProgressBar, Stack, Text } from '@drizztdourden08/tessera';
import { METER_TONE, NO_DATA } from '../accounts.constants';
import type { UsageCellProps } from '../accounts.type';
import { resetText } from '../model';

const toneOf = (severity: string | undefined) => METER_TONE[severity === 'critical' || severity === 'warning' ? severity : 'normal'];

const UsageCell = (props: UsageCellProps) => {
  const { limit, label, now } = props;
  if (typeof limit?.percent !== 'number') return <Text variant="caption" tone="faint" className="accounts-strip__cell">{NO_DATA}</Text>;
  const percent = Math.round(limit.percent);
  return (
    <Stack gap="2xs" className="accounts-strip__cell">
      <ProgressBar value={percent} max={100} tone={toneOf(limit.severity)} label={`${label} use`} formatValue={(value) => `${value}%`} showValue />
      <Text variant="caption" tone="muted">{resetText(limit.resetsAt, now)}</Text>
    </Stack>
  );
};

export { UsageCell };
