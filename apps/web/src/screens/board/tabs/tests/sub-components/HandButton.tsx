/* @layer renderer-app @kind component */
import { useTestActions } from '../behavior/useTestActions';
import { useTestsShared } from '../behavior/useTestsShared';
import type { HandButtonProps } from '../tests-props.type';
import { TipButton } from './TipButton';

const HandButton = (props: HandButtonProps) => {
  const { run, handed } = props;
  const actions = useTestActions(useTestsShared().boardKey);
  return (
    <TipButton
      icon="hand"
      label={handed ? `Stop handing run ${run.id} to you` : `Hand run ${run.id} to you`}
      tip={handed ? 'Unhand: stop handing this run to you' : 'Hand to me'}
      active={handed}
      disabled={actions.busy}
      onClick={() => void actions.hand(run, handed)}
    />
  );
};

export { HandButton };
