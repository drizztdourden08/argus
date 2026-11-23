/* @layer renderer-app @kind component */
import { Stack, Text } from '@drizztdourden08/tessera';
import { NO_USAGE } from '../accounts.constants';
import type { LimitMetersProps } from '../cards.type';
import { reportedLimits } from '../model';
import { LimitMeter } from './LimitMeter';

const LimitMeters = (props: LimitMetersProps) => {
  const { account, now } = props;
  const limits = reportedLimits(account);
  if (limits.length === 0) return <Text variant="caption" tone="faint">{NO_USAGE}</Text>;
  return (
    <Stack gap="sm">
      {limits.map((limit) => <LimitMeter key={`${limit.kind}|${limit.model ?? ''}`} limit={limit} now={now} />)}
    </Stack>
  );
};

export { LimitMeters };
