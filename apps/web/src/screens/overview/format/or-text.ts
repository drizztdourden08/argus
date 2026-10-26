/* @layer renderer-app @kind logic */
const orText = (value: string | null | undefined, fallback: string): string =>
  value === undefined || value === null || value === '' ? fallback : value;

export { orText };
