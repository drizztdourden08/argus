/* @layer renderer-app @kind test */
const snapshot = (scope: string, coll: string, docs: Record<string, Readonly<Record<string, unknown>>>, seq = 10) => ({
  scope,
  seq,
  coll,
  docs: Object.entries(docs).map(([id, data]) => ({ id, data })),
});

export { snapshot };
