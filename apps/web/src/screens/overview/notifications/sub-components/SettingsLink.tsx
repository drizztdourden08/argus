/* @layer renderer-app @kind component */
import { Button } from '@drizztdourden08/tessera';
import { useAppSettings } from '../../../../shell/AppSettings';

const SettingsLink = () => {
  const { openSettings } = useAppSettings();
  return (
    <Button size="sm" onClick={() => openSettings('notifications')}>
      Notification settings
    </Button>
  );
};

export { SettingsLink };
