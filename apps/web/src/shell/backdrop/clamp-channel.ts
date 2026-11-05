/* @layer renderer-app @kind logic */
const clampChannel = (value: number): number => Math.min(255, Math.max(0, Math.round(value)));

export { clampChannel };
