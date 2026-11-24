/* @layer renderer-app @kind component */
import { Inline, Stack, Text } from '@drizztdourden08/tessera';
import { nextPair, whenText } from '../model';
import { SLOT_FROM, SLOT_TO, STRIP_LABEL } from '../next.constants';
import type { NextStripProps } from '../next.type';
import { AccountSlot } from './AccountSlot';
import { NextRun } from './NextRun';

const NextStrip = (props: NextStripProps) => {
  const { accounts, status, actions } = props;
  const { from, to, when, phase } = nextPair(status);
  return (
    <Inline gap="md" align="center" wrap role="group" aria-label={STRIP_LABEL}>
      <AccountSlot label={SLOT_FROM} accounts={accounts} value={from} disabled={actions.busy} onPick={(key) => actions.choose(key, to)} onClear={actions.clear} />
      <Stack gap="xs" align="center">
        <NextRun phase={phase} />
        <Text variant="caption" tone="muted">{whenText(when)}</Text>
      </Stack>
      <AccountSlot label={SLOT_TO} accounts={accounts} value={to} disabled={actions.busy} onPick={(key) => actions.choose(from, key)} onClear={actions.clear} />
    </Inline>
  );
};

export { NextStrip };
