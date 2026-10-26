/* @layer renderer-app @kind logic */
const byCreated = <T extends { id: string; createdAt?: string }>(a: T, b: T): number =>
  String(a.createdAt).localeCompare(String(b.createdAt)) || a.id.localeCompare(b.id, undefined, { numeric: true });

export { byCreated };
