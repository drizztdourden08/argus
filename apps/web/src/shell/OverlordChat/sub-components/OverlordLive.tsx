/* @layer renderer-app @kind component */
import { Callout, EmptyState, Stack } from '@drizztdourden08/tessera';
import { useOverlordPanel } from '../behavior/useOverlordPanel';
import { READING } from '../OverlordPanel.constants';
import { ApprovalStrip } from './ApprovalStrip';
import { ModeSection } from './ModeSection';
import { SpendSection } from './SpendSection';
import { StatusSection } from './StatusSection';

const OverlordLive = () => {
  const panel = useOverlordPanel();
  const { status, spend } = panel;
  if (!status) return panel.problem ? <Callout tone="danger">{`The Overlord could not be read: ${panel.problem}.`}</Callout> : <EmptyState message={READING} />;
  return (
    <Stack gap="md">
      <StatusSection status={status} action={panel.control} onControl={panel.onControl} />
      <ApprovalStrip approvals={panel.pending} action={panel.decision} onDecide={panel.onDecide} />
      <ModeSection mode={status.mode} action={panel.modeAction} onSwitch={panel.onSwitch} />
      {spend && <SpendSection spend={spend} />}
    </Stack>
  );
};

export { OverlordLive };
