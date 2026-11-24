/* @layer renderer-app @kind component */
import { Callout, Dialog, SettingsSection, Stack } from '@drizztdourden08/tessera';
import { useState } from 'react';
import type { OverlordMode } from '../../../data/overlord';
import { asMode } from '../behavior/as-mode';
import { needsConfirm } from '../behavior/needs-confirm';
import { modeSteps } from '../behavior/mode-steps';
import { MODE_CONFIRM, MODE_NOTES } from '../OverlordChat.constants';
import type { ModeSectionProps } from '../OverlordChat.type';

const ModeSection = (props: ModeSectionProps) => {
  const { mode, action, onSwitch } = props;
  const [asked, setAsked] = useState<OverlordMode | null>(null);
  const current = asMode(mode);
  const steps = modeSteps(current);
  const options = steps.map((step) => ({ value: step.mode, label: step.label, disabled: step.reason ?? false }));
  const reasons = steps.filter((step) => step.reason).map((step) => `${step.label}: ${step.reason}.`);
  const choose = (next: OverlordMode) => {
    if (next === current || action.busy) return;
    if (needsConfirm(current, next)) setAsked(next);
    else onSwitch(next);
  };
  const row = {
    id: 'mode',
    title: 'Mode',
    description: [MODE_NOTES[current], ...reasons].join(' '),
    input: { kind: 'segmented' as const, value: current, options, onChange: (next: string) => choose(next as OverlordMode) },
  };
  return (
    <Stack gap="sm">
      <SettingsSection title="Mode" rows={[row]} />
      {action.problem && <Callout tone="danger">{action.problem}</Callout>}
      <Dialog
        open={asked !== null}
        size="md"
        title={`Switch to ${asked ?? ''}`}
        message={MODE_CONFIRM}
        confirmLabel="Switch"
        confirmDisabled={action.busy}
        onConfirm={() => {
          if (asked) onSwitch(asked);
          setAsked(null);
        }}
        onCancel={() => setAsked(null)}
      />
    </Stack>
  );
};

export { ModeSection };
