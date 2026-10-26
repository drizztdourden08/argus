/* @layer renderer-app @kind logic */
const compareIds = (left: string, right: string): number => left.localeCompare(right, undefined, { numeric: true });

export { compareIds };
