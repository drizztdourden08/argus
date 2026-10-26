/* @layer renderer-app @kind logic */
const errorMessage = (error: unknown): string => (error instanceof Error ? error.message : 'unknown error');

export { errorMessage };
