/* @layer renderer-app @kind component */
import { Flex, Icon, IconButton } from '@drizztdourden08/tessera';
import { useOwnerProfile } from '../../../owner';
import { useAppSettings } from '../../AppSettings';
import { OwnerMark } from '../../OwnerMark';

const SettingsEntries = () => {
  const { openSettings } = useAppSettings();
  const { name } = useOwnerProfile();
  return (
    <Flex gap="xs" align="center">
      <IconButton label={`${name}, profile`} variant="ghost" onClick={() => openSettings('profile')}>
        <OwnerMark size="sm" label={`${name}, profile`} />
      </IconButton>
      <IconButton label="Settings" variant="ghost" onClick={() => openSettings()}>
        <Icon name="settings" />
      </IconButton>
    </Flex>
  );
};

export { SettingsEntries };
