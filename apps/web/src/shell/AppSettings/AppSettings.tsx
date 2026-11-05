/* @layer renderer-app @kind component */
import type { AppSettingsProviderProps } from './AppSettings.type';
import { AppSettingsContext } from './behavior/app-settings-context';
import { useAppSettingsState } from './behavior/useAppSettingsState';
import { AppSettingsDialog } from './sub-components/AppSettingsDialog';

const AppSettingsProvider = (props: AppSettingsProviderProps) => {
  const api = useAppSettingsState();
  return (
    <AppSettingsContext.Provider value={api}>
      {props.children}
      {api.open && <AppSettingsDialog section={api.section} onSection={api.setSection} onClose={api.close} />}
    </AppSettingsContext.Provider>
  );
};

export { AppSettingsProvider };
