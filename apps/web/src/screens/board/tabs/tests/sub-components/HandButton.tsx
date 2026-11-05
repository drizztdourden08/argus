/* @layer renderer-app @kind component */
import { Button } from '@drizztdourden08/tessera';
import { useTestActions } from '../behavior/useTestActions';
import { useTestsShared } from '../behavior/useTestsShared';
import type { HandButtonProps } from '../tests-props.type';

const HandButton = (props: HandButtonProps) => {
  const { run, handed } = props;
  const actions = useTestActions(useTestsShared().boardKey);
  const label = handed ? `Stop handing run ${run.id} to you` : `Hand run ${run.id} to you`;
  return (
    <Button size="sm" variant={handed ? 'quiet' : 'tertiary'} disabled={actions.busy} aria-label={label} onClick={() => void actions.hand(run, handed)}>
      {handed ? 'Unhand' : 'Hand to me'}
    </Button>
  );
};

export { HandButton };
