/* @layer renderer-app @kind logic */
const byRank = <T extends { id: string; rank?: number; createdAt?: string }>(a: T, b: T): number =>
  (a.rank ?? 0) - (b.rank ?? 0) || String(a.createdAt).localeCompare(String(b.createdAt)) || a.id.localeCompare(b.id, undefined, { numeric: true });

export { byRank };
