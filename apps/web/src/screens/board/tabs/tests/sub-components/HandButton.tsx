/* @layer renderer-app @kind component */
import { Button } from '@drizztdourden08/tessera';
import { useTestActions } from '../behavior/useTestActions';
import { useTestsShared } from '../behavior/useTestsShared';
import type { HandButtonProps } from '../tests-props.type';

const HandButton = (props: HandButtonProps) => {
  const { tag, handed } = props;
  const actions = useTestActions(useTestsShared().boardKey);
  const label = handed ? `Stop handing build ${tag} to you` : `Hand build ${tag} to you`;
  return (
    <Button size="sm" variant={handed ? 'quiet' : 'tertiary'} disabled={actions.busy} aria-label={label} onClick={() => void actions.hand(handed ? null : tag)}>
      {handed ? 'Unhand' : 'Hand to me'}
    </Button>
  );
};

export { HandButton };
