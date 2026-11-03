/* @layer renderer-app @kind logic */
const depRef = (end: { project: string; item: string | null }): string => `${end.project}/${end.item ?? '?'}`;

export { depRef };
