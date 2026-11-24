/* @layer renderer-app @kind logic */
import { RENAMED_THREADS } from './canonical-thread.constants';

const canonicalThread = (thread: string): string => RENAMED_THREADS[thread] ?? thread;

export { canonicalThread };
