/* @layer renderer-app @kind component */
import { Inline } from '@drizztdourden08/tessera';
import { useState } from 'react';
import { clearSlot, copyDirection, placeSlot } from '../model';
import { DIRECTION_LTR, EMPTY_PAIR, SLOT_LEFT, SLOT_RIGHT, SYNC_PAIR_LABEL } from '../next.constants';
import type { Direction, SlotPair, SyncChoicesProps } from '../next.type';
import { AccountSlot } from './AccountSlot';
import { SyncArrow } from './SyncArrow';

const SyncChoices = (props: SyncChoicesProps) => {
  const { accounts, disabled, actions, onReview } = props;
  const [pair, setPair] = useState<SlotPair>(EMPTY_PAIR);
  const [direction, setDirection] = useState<Direction>(DIRECTION_LTR);
  const { left, right } = pair;
  const go = () => {
    const copy = left && right ? copyDirection(direction, left, right) : null;
    if (copy) actions.copy(copy.from, copy.to);
    else onReview();
  };
  return (
    <Inline gap="lg" align="center" justify="center" wrap role="group" aria-label={SYNC_PAIR_LABEL}>
      <AccountSlot label={SLOT_LEFT} accounts={accounts} value={left} disabled={disabled} onPick={(key) => setPair(placeSlot(pair, 'left', key))} onClear={() => setPair(clearSlot(pair, 'left'))} />
      <SyncArrow direction={direction} ready={left !== null && right !== null} disabled={disabled} onDirection={setDirection} onGo={go} />
      <AccountSlot label={SLOT_RIGHT} accounts={accounts} value={right} disabled={disabled} onPick={(key) => setPair(placeSlot(pair, 'right', key))} onClear={() => setPair(clearSlot(pair, 'right'))} />
    </Inline>
  );
};

export { SyncChoices };
