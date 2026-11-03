/* @layer renderer-app @kind logic */
const openKeys = (branches: readonly string[], closed: ReadonlySet<string>): string[] => branches.filter((key) => !closed.has(key));

export { openKeys };
