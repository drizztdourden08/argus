/* @layer renderer-app @kind component */
import { Button, Inline, ProgressBar, Stack, Status, Text } from '@drizztdourden08/tessera';
import type { SyncRunLineProps } from '../accounts.type';
import { useQuitCancel } from '../behavior/useQuitCancel';
import { PHASE_LABEL, QUIT_CANCEL_LABEL, QUIT_FALLBACK } from '../compare.constants';
import { recentSteps, stepLine } from '../model';

const SyncRunLine = (props: SyncRunLineProps) => {
  const { run, steps, accounts } = props;
  const cancel = useQuitCancel();
  const phase = run.phase ?? 'idle';
  const total = run.total ?? 0;
  const quitting = phase === 'waiting_for_quit';
  return (
    <Stack gap="2xs">
      <Inline gap="sm" align="center" wrap>
        <Status tone={quitting ? 'warning' : 'info'} size="sm" dot pulse role="status">{PHASE_LABEL[phase]}</Status>
        <Text variant="caption" tone={quitting ? 'warning' : 'muted'}>{quitting ? (run.detail ?? QUIT_FALLBACK) : (run.detail ?? '')}</Text>
        {quitting && <Button size="xs" variant="secondary" disabled={cancel.busy} onClick={cancel.run}>{QUIT_CANCEL_LABEL}</Button>}
      </Inline>
      <ProgressBar value={run.step ?? 0} max={Math.max(total, 1)} indeterminate={total === 0} label="Sync progress" showValue={total > 0} live />
      {recentSteps(steps).map((step) => (
        <Text key={step.id} variant="caption" tone={step.status === 'failed' ? 'danger' : 'muted'}>{stepLine(step, accounts)}</Text>
      ))}
    </Stack>
  );
};

export { SyncRunLine };
