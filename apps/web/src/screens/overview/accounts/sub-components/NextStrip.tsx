/* @layer renderer-app @kind component */
import { Inline, Stack, Stepper, Text } from '@drizztdourden08/tessera';
import { nextSteps, whenText } from '../model';
import { SLOT_FROM, SLOT_TO, STRIP_LABEL } from '../next.constants';
import type { NextStripProps } from '../next.type';
import { AccountSlot } from './AccountSlot';

const NextStrip = (props: NextStripProps) => {
  const { accounts, status, actions } = props;
  const from = status?.next?.from ?? null;
  const to = status?.next?.to ?? null;
  const view = nextSteps(status?.run?.phase);
  return (
    <Inline gap="md" align="center" wrap role="group" aria-label={STRIP_LABEL}>
      <AccountSlot label={SLOT_FROM} accounts={accounts} value={from} disabled={actions.busy} onPick={(key) => actions.choose(key, to)} onClear={actions.clear} />
      <Stack gap="xs" align="center">
        <Stepper {...view} compact size="sm" label={STRIP_LABEL} />
        <Text variant="caption" tone="muted">{whenText(status?.next?.when)}</Text>
      </Stack>
      <AccountSlot label={SLOT_TO} accounts={accounts} value={to} disabled={actions.busy} onPick={(key) => actions.choose(from, key)} onClear={actions.clear} />
    </Inline>
  );
};

export { NextStrip };
