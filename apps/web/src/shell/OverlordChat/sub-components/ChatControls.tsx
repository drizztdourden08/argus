/* @layer renderer-app @kind component */
import { Button, Flex, Text } from '@drizztdourden08/tessera';
import { controlOverlord, useAction } from '../../../data/overlord';
import { useAppSettings } from '../../AppSettings';
import { isRunning } from '../behavior/is-running';
import { overlordNote } from '../behavior/overlord-note';
import { useOverlordStatus } from '../behavior/useOverlordStatus';
import { KEY_SETTINGS_LABEL, NO_KEY_PATTERN, RUNNING_WORD } from '../OverlordChat.constants';

const ChatControls = () => {
  const status = useOverlordStatus();
  const action = useAction(status.reload);
  const { openSettings } = useAppSettings();
  const state = status.data?.status;
  const running = isRunning(state);
  const note = running ? RUNNING_WORD : overlordNote(status.data);
  const needsKey = !running && NO_KEY_PATTERN.test(note);
  return (
    <Flex gap="sm" align="center">
      <Text variant="caption" tone="muted">{note}</Text>
      {needsKey && <Button size="sm" onClick={() => openSettings('system')}>{KEY_SETTINGS_LABEL}</Button>}
      <Button size="sm" disabled={action.busy || state === undefined} onClick={() => void action.run(() => controlOverlord('restart'))}>Restart</Button>
    </Flex>
  );
};

export { ChatControls };
