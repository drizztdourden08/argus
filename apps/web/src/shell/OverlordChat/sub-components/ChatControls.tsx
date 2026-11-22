/* @layer renderer-app @kind component */
import { Button, Flex, Text } from '@drizztdourden08/tessera';
import { controlOverlord, useAction } from '../../../data/overlord';
import { isRunning } from '../behavior/is-running';
import { overlordNote } from '../behavior/overlord-note';
import { useOverlordStatus } from '../behavior/useOverlordStatus';
import { RUNNING_WORD } from '../OverlordChat.constants';
import { KeyButton } from './KeyButton';

const ChatControls = () => {
  const status = useOverlordStatus();
  const action = useAction(status.reload);
  const state = status.data?.status;
  const running = isRunning(state);
  const note = running ? RUNNING_WORD : overlordNote(status.data);
  return (
    <Flex gap="sm" align="center">
      <Text variant="caption" tone="muted">{note}</Text>
      {!running && <KeyButton note={note} />}
      <Button size="sm" disabled={action.busy || state === undefined} onClick={() => void action.run(() => controlOverlord('restart'))}>Restart</Button>
    </Flex>
  );
};

export { ChatControls };
