/* @layer renderer-app @kind logic */
const closedAfter = (branches: readonly string[], expanded: readonly string[]): Set<string> =>
  new Set(branches.filter((key) => !expanded.includes(key)));

export { closedAfter };
