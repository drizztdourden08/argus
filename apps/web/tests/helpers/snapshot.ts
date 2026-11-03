/* @layer renderer-app @kind test */
const snapshot = (scope: string, coll: string, docs: Readonly<Record<string, object>>, seq = 10) => ({
  scope,
  seq,
  coll,
  docs: Object.entries(docs).map(([id, data]) => ({ id, data: data as Readonly<Record<string, unknown>> })),
});

export { snapshot };
