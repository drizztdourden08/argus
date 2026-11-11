/* @layer renderer-app @kind logic */
const toMs = (value: number | string | null | undefined): number | null => {
  if (value === null || value === undefined) return null;
  const ms = typeof value === 'number' ? value : Date.parse(value);
  return Number.isNaN(ms) ? null : ms;
};

export { toMs };
