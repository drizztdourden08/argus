/* @layer renderer-app @kind logic */
const nonEmpty = (value: string | null | undefined, fallback: string): string => (value === undefined || value === null || value === '' ? fallback : value);

export { nonEmpty };
