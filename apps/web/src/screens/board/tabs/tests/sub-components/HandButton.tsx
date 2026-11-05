/* @layer renderer-app @kind component */
import { Icon, IconButton } from '@drizztdourden08/tessera';
import { itemCode } from '../../../../../shell';
import { useTestActions } from '../behavior/useTestActions';
import { useTestsShared } from '../behavior/useTestsShared';
import type { HandButtonProps } from '../tests-props.type';

const HandButton = (props: HandButtonProps) => {
  const { run, handed } = props;
  const actions = useTestActions(useTestsShared().boardKey);
  const code = itemCode(run);
  return (
    <IconButton
      size="xs"
      label={handed ? `Stop handing run ${code} to you` : `Hand run ${code} to you`}
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
