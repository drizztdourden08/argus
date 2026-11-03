/* @layer renderer-app @kind logic */
const slashed = (path: string): string => path.replace(/\\/g, '/').replace(/\/+$/, '');

export { slashed };
