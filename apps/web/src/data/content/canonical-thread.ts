/* @layer renderer-app @kind logic */
const RENAMED_THREADS: Readonly<Record<string, string>> = { coordinator: 'lead', 'coordinator-hosted': 'lead-hosted' };

const canonicalThread = (thread: string): string => RENAMED_THREADS[thread] ?? thread;

export { canonicalThread };
