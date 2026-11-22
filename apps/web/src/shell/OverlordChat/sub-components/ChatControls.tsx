/* @layer renderer-app @kind component */
import { Button, Flex } from '@drizztdourden08/tessera';
import { controlOverlord, useAction } from '../../../data/overlord';
import type { OverlordAction } from '../../../data/overlord';
import { isRunning } from '../behavior/is-running';
import { useOverlordStatus } from '../behavior/useOverlordStatus';

const ChatControls = () => {
  const status = useOverlordStatus();
  const action = useAction(status.reload);
  const state = status.data?.status;
  const running = isRunning(state);
  const live = running || state === 'starting';
  const send = (verb: OverlordAction) => void action.run(() => controlOverlord(verb));
  return (
    <Flex gap="sm">
      <Button size="sm" variant="primary" disabled={action.busy || state === undefined || live} onClick={() => send('start')}>Start</Button>
      <Button size="sm" disabled={action.busy || !live} onClick={() => send('stop')}>Stop</Button>
    </Flex>
  );
};

export { ChatControls };
