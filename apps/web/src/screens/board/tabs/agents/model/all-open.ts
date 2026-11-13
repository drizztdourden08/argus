/* @layer renderer-app @kind logic */
const allOpen = (open: ReadonlySet<string>, ids: readonly string[]): boolean => ids.length > 0 && ids.every((id) => open.has(id));

export { allOpen };
