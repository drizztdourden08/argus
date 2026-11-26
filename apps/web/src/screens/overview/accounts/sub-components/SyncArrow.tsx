/* @layer renderer-app @kind component */
import { Button, SegmentedControl, Stack, Text } from '@drizztdourden08/tessera';
import { DIRECTION_BOTH, DIRECTION_LABEL, DIRECTION_OPTIONS, DIRECTION_TEXT, SYNC_GO_COPY, SYNC_GO_REVIEW } from '../next.constants';
import type { SyncArrowProps } from '../next.type';

const SyncArrow = (props: SyncArrowProps) => {
  const { direction, ready, disabled, onDirection, onGo } = props;
  const both = direction === DIRECTION_BOTH;
  return (
    <Stack gap="xs" align="center">
      <SegmentedControl aria-label={DIRECTION_LABEL} size="sm" value={direction} options={DIRECTION_OPTIONS} onChange={onDirection} />
      <Text variant="caption" tone="muted">{DIRECTION_TEXT[direction]}</Text>
      <Button size="sm" variant="primary" disabled={disabled || (!both && !ready)} onClick={onGo}>{both ? SYNC_GO_REVIEW : SYNC_GO_COPY}</Button>
    </Stack>
  );
};

export { SyncArrow };
