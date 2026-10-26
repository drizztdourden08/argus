/* @layer renderer-app @kind logic */
const isOverCap = (memory: number, cap: number | null): boolean => cap !== null && cap > 0 && memory > cap;

export { isOverCap };
