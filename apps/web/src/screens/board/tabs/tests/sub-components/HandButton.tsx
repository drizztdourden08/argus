/* @layer renderer-app @kind component */
import { Icon, IconButton } from '@drizztdourden08/tessera';
import { useTestActions } from '../behavior/useTestActions';
import { useTestsShared } from '../behavior/useTestsShared';
import type { HandButtonProps } from '../tests-props.type';

const HandButton = (props: HandButtonProps) => {
  const { run, handed } = props;
  const actions = useTestActions(useTestsShared().boardKey);
  return (
    <IconButton
      size="xs"
      label={handed ? `Stop handing run ${run.id} to you` : `Hand run ${run.id} to you`}
      tooltip={handed ? 'Unhand: stop handing this run to you' : 'Hand to me'}
      active={handed}
      disabled={actions.busy}
      onClick={() => void actions.hand(run, handed)}
    >
      <Icon name="hand" />
    </IconButton>
  );
};

export { HandButton };
