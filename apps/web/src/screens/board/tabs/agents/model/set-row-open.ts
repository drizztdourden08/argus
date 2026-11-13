/* @layer renderer-app @kind logic */
const setRowOpen = (open: ReadonlySet<string>, id: string, value: boolean): ReadonlySet<string> => {
  const next = new Set(open);
  if (value) next.add(id);
  else next.delete(id);
  return next;
};

export { setRowOpen };
