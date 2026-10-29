/* @layer renderer-app @kind component */
import { Text } from '@drizztdourden08/tessera';
import { RecordDrawer } from '../../../shared';
import type { StepLogDrawerProps } from '../changes.type';
import { StepLogBody } from './StepLogBody';

const StepLogDrawer = (props: StepLogDrawerProps) => {
  const { boardKey, target, onClose } = props;
  return (
    <RecordDrawer open={target !== null} label="Check step log" wide={false} eyebrow={target?.runId} title={target?.name ?? ''} onClose={onClose}>
      {target && (
        <>
          <Text variant="caption" tone="muted" mono>{target.command}</Text>
          <StepLogBody key={`${target.runId}-${target.index}`} boardKey={boardKey} target={target} />
        </>
      )}
    </RecordDrawer>
  );
};

export { StepLogDrawer };
