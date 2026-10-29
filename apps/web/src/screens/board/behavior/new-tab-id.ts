/* @layer renderer-app @kind logic */
const newTabId = (): string => Math.random().toString(36).slice(2, 10);

export { newTabId };
