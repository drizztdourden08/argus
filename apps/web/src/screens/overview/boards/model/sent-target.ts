/* @layer renderer-app @kind logic */
const sentTarget = (project: string | undefined, nameOf: (key: string) => string): string =>
  project === 'all' ? 'all projects' : nameOf(project ?? '');

export { sentTarget };
