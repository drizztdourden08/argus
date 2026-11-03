/* @layer renderer-app @kind logic */
import { TAB_ID_BYTES } from './hub-tab-id.constants';

const newHubTabId = (): string =>
  Array.from(crypto.getRandomValues(new Uint8Array(TAB_ID_BYTES)), (byte) => byte.toString(16).padStart(2, '0')).join('');

export { newHubTabId };
