/* @layer renderer-app @kind component */
import { Stack, Text } from '@drizztdourden08/tessera';
import { useCountdown } from '../behavior/useCountdown';
import { atZeroText, countdownText, toMs } from '../model';
import { NO_REASON, STAGE_PLAN, ZERO_GRACE_MS, ZERO_TEXT } from '../switch.constants';
import type { ProcessDocProps } from '../switch.type';

const CountdownLine = (props: ProcessDocProps) => {
  const { doc } = props;
  const endsAt = toMs(doc.countdownEndsAt);
  const left = useCountdown(endsAt);
  if (endsAt === null) return <Text tone={doc.blocker ? 'warning' : undefined}>{doc.blocker ?? STAGE_PLAN.countdown}</Text>;
  return (
    <Stack gap="2xs">
      <Text role="timer" aria-live="off">
        <Text as="span" weight={700} numeric>{countdownText(left)}</Text>
        {` left. ${atZeroText(doc)}`}
      </Text>
      {left <= -ZERO_GRACE_MS && <Text variant="caption" tone="warning" role="status">{`${ZERO_TEXT} ${doc.blocker ?? NO_REASON}`}</Text>}
    </Stack>
  );
};

export { CountdownLine };
