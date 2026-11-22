/* @layer renderer-app @kind component */
import { Button } from '@drizztdourden08/tessera';
import { useAppSettings } from '../../AppSettings';
import { KEY_SETTINGS_LABEL, NO_KEY_PATTERN } from '../OverlordChat.constants';

const KeyButton = ({ note }: { note: string }) => {
  const { openSettings } = useAppSettings();
  if (!NO_KEY_PATTERN.test(note)) return null;
  return <Button size="sm" onClick={() => openSettings('system')}>{KEY_SETTINGS_LABEL}</Button>;
};

export { KeyButton };
