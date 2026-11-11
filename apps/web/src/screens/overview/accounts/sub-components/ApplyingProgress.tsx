/* @layer renderer-app @kind component */
import { ProgressBar, Text } from '@drizztdourden08/tessera';
import { PHASE_LABEL } from '../compare.constants';
import type { ApplyingProgressProps } from '../compare.type';

const ApplyingProgress = (props: ApplyingProgressProps) => {
  const { run } = props;
  const phase = run?.phase ?? 'idle';
  const total = run?.total ?? 0;
  return (
    <>
      <Text size="md" weight={600}>{`Syncing: ${PHASE_LABEL[phase]}`}</Text>
      <ProgressBar value={run?.step ?? 0} max={Math.max(total, 1)} indeterminate={total === 0 && phase !== 'done'} tone={phase === 'failed' ? 'danger' : 'primary'} showValue={total > 0} live />
    </>
  );
};

export { ApplyingProgress };
