/* @layer renderer-app @kind component */
import { Button } from '@drizztdourden08/tessera';
import { useTestActions } from '../behavior/useTestActions';
import type { RerunButtonProps } from '../tests-props.type';

const RerunButton = (props: RerunButtonProps) => {
  const { test, boardKey } = props;
  const actions = useTestActions(boardKey);
  if (test.status === 'pending') return null;
  return (
    <Button size="sm" variant="ghost" disabled={actions.busy} aria-label={`Run ${test.id} again`} onClick={() => void actions.rerun(test.id)}>
      Run again
    </Button>
  );
};

export { RerunButton };
