/* @layer renderer-app @kind logic */
const BYTES = 8;

const newHubTabId = (): string =>
  Array.from(crypto.getRandomValues(new Uint8Array(BYTES)), (byte) => byte.toString(16).padStart(2, '0')).join('');

export { newHubTabId };
