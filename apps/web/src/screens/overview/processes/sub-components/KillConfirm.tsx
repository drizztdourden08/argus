/* @layer renderer-app @kind component */
import { Button, ButtonRow, Callout } from '@drizztdourden08/tessera';
import type { KillConfirmProps } from '../processes.type';

const KillConfirm = (props: KillConfirmProps) => {
  const { question, busy, onKill, onKillTree, onCancel } = props;
  return (
    <Callout tone="danger" action={(
      <ButtonRow>
        <Button size="sm" variant="danger" disabled={busy} onClick={onKill}>Kill</Button>
        {onKillTree && <Button size="sm" variant="danger" disabled={busy} onClick={onKillTree}>Kill with children</Button>}
        <Button size="sm" onClick={onCancel}>Cancel</Button>
      </ButtonRow>
    )}
    >
      {question}
    </Callout>
  );
};

export { KillConfirm };
