/* @layer renderer-app @kind test */
const MS_PER_MINUTE = 60_000;

const minutesAgo = (minutes: number): string => new Date(Date.now() - minutes * MS_PER_MINUTE).toISOString();

export { minutesAgo };
