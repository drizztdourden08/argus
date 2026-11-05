/* @layer renderer-app @kind logic */
import { useContext } from 'react';
import type { AppSettingsApi } from '../AppSettings.type';
import { AppSettingsContext } from './app-settings-context';

const useAppSettings = (): AppSettingsApi => {
  const api = useContext(AppSettingsContext);
  if (!api) throw new Error('useAppSettings needs an AppSettingsProvider.');
  return api;
};

export { useAppSettings };
