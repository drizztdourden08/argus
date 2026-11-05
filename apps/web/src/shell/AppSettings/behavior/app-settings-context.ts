/* @layer renderer-app @kind logic */
import { createContext } from 'react';
import type { AppSettingsApi } from '../AppSettings.type';

const AppSettingsContext = createContext<AppSettingsApi | null>(null);

export { AppSettingsContext };
