/* @layer renderer-app @kind component */
import { Callout, Dialog, SegmentedControl, Stack, Text } from '@drizztdourden08/tessera';
import { useState } from 'react';
import type { OverlordMode } from '../../../../data/overlord';
import { Panel } from '../../../../shell';
import { modeSteps } from '../model';
import { MODE_CONFIRM, MODE_NOTES, asMode, needsConfirm } from '../overlord.constants';
import type { ModeCardProps } from '../overlord.type';

const ModeCard = (props: ModeCardProps) => {
  const { mode, action, onSwitch } = props;
  const [asked, setAsked] = useState<OverlordMode | null>(null);
  const current = asMode(mode);
  const steps = modeSteps(current);
  const options = steps.map((step) => ({ value: step.mode, label: step.label, disabled: step.reason ?? false }));
  const choose = (next: OverlordMode) => (needsConfirm(current, next) ? setAsked(next) : onSwitch(next));
  const reasons = steps.filter((step) => step.reason).map((step) => `${step.label}: ${step.reason}.`);
  return (
    <Panel title="Mode">
      <Stack gap="sm">
        <SegmentedControl aria-label="Overlord mode" selectedTone="neutral" value={current} options={options} disabled={action.busy} onChange={(next) => next !== current && choose(next)} />
        <Text variant="caption" tone="muted">{MODE_NOTES[current]}</Text>
        {reasons.length > 0 && <Text variant="caption" tone="muted">{reasons.join(' ')}</Text>}
        {action.problem && <Callout tone="danger">{action.problem}</Callout>}
      </Stack>
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
    </Panel>
  );
};

export { ModeCard };
