/* @layer renderer-app @kind constants */
import type { AccountsSettingsState } from './accounts-settings.type';

const SETTINGS_PATH = '/api/accounts/settings';

const EMPTY: AccountsSettingsState = { settings: null, problem: null, busy: false };

export { EMPTY, SETTINGS_PATH };
