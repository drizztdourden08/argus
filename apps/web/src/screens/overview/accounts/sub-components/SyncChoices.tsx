/* @layer renderer-app @kind component */
import { Button, Inline, SegmentedControl, Stack } from '@drizztdourden08/tessera';
import { useState } from 'react';
import { copyDirection } from '../model';
import { DIRECTION_BOTH, DIRECTION_LABEL, DIRECTION_LTR, DIRECTION_OPTIONS, SLOT_LEFT, SLOT_RIGHT, SYNC_GO_COPY, SYNC_GO_REVIEW } from '../next.constants';
import type { Direction, SyncChoicesProps } from '../next.type';
import { AccountSlot } from './AccountSlot';

const SyncChoices = (props: SyncChoicesProps) => {
  const { accounts, disabled, actions, onReview } = props;
  const [left, setLeft] = useState<string | null>(null);
  const [right, setRight] = useState<string | null>(null);
  const [direction, setDirection] = useState<Direction>(DIRECTION_LTR);
  const ready = left !== null && right !== null && left !== right;
  const go = () => {
    const pair = ready ? copyDirection(direction, left, right) : null;
    if (pair) actions.copy(pair.from, pair.to);
    else onReview();
  };
  return (
    <Stack gap="sm">
      <Inline gap="md" align="center" wrap>
        <AccountSlot label={SLOT_LEFT} accounts={accounts} value={left} disabled={disabled} onPick={setLeft} />
        <SegmentedControl aria-label={DIRECTION_LABEL} size="sm" value={direction} options={[...DIRECTION_OPTIONS]} onChange={(value) => setDirection(value as Direction)} />
        <AccountSlot label={SLOT_RIGHT} accounts={accounts} value={right} disabled={disabled} onPick={setRight} />
      </Inline>
      <Inline gap="xs">
        <Button size="sm" variant="primary" disabled={disabled || (direction !== DIRECTION_BOTH && !ready)} onClick={go}>{direction === DIRECTION_BOTH ? SYNC_GO_REVIEW : SYNC_GO_COPY}</Button>
      </Inline>
    </Stack>
  );
};

export { SyncChoices };
