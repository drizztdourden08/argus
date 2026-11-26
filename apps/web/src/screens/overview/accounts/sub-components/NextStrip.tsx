/* @layer renderer-app @kind component */
import { Inline } from '@drizztdourden08/tessera';
import { clearSlot, nextPair, placeSlot, slotHint } from '../model';
import { SLOT_FROM, SLOT_TO, STRIP_LABEL } from '../next.constants';
import type { NextStripProps } from '../next.type';
import { AccountSlot } from './AccountSlot';
import { NextArrow } from './NextArrow';

const NextStrip = (props: NextStripProps) => {
  const { accounts, status, actions } = props;
  const { picked, auto, when, phase } = nextPair(status);
  return (
    <Inline gap="lg" align="center" justify="center" wrap role="group" aria-label={STRIP_LABEL}>
      <AccountSlot
        label={SLOT_FROM}
        accounts={accounts}
        value={picked.left}
        hint={slotHint(accounts, picked.left, auto.left)}
        disabled={actions.busy}
        onPick={(key) => actions.save(placeSlot(picked, 'left', key))}
        onClear={() => actions.save(clearSlot(picked, 'left'))}
      />
      <NextArrow phase={phase} when={when} />
      <AccountSlot
        label={SLOT_TO}
        accounts={accounts}
        value={picked.right}
        hint={slotHint(accounts, picked.right, auto.right)}
        disabled={actions.busy}
        onPick={(key) => actions.save(placeSlot(picked, 'right', key))}
        onClear={() => actions.save(clearSlot(picked, 'right'))}
      />
    </Inline>
  );
};

export { NextStrip };
