/* @layer renderer-app @kind component */
import { Button, Inline } from '@drizztdourden08/tessera';
import { useSwitchActions } from '../behavior/useSwitchActions';
import { canCancel, canSkip, wizardSteps } from '../model';
import { CANCEL_LABEL, NOW_LABEL, SKIP_STEP_LABEL } from '../switch.constants';
import type { ProcessDocProps } from '../switch.type';

const ProcessActions = (props: ProcessDocProps) => {
  const { doc } = props;
  const actions = useSwitchActions();
  const stage = wizardSteps(doc).currentId;
  return (
    <Inline gap="xs">
      {doc.phase === 'countdown' && <Button size="xs" disabled={actions.busy} onClick={() => actions.start(doc.to ?? '', false)}>{NOW_LABEL}</Button>}
      {canSkip(doc, stage) && <Button size="xs" variant="secondary" disabled={actions.busy} onClick={() => actions.skipStep(stage)}>{SKIP_STEP_LABEL}</Button>}
      {canCancel(doc) && <Button size="xs" variant="ghost" disabled={actions.busy} onClick={actions.cancel}>{CANCEL_LABEL}</Button>}
    </Inline>
  );
};

export { ProcessActions };
