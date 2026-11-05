/* @layer renderer-app @kind logic */
const boardUrl = (key: string, hash = ''): string => `/p/${encodeURIComponent(key)}/${hash}`;

export { boardUrl };
