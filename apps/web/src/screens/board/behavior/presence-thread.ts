/* @layer renderer-app @kind logic */
import { THREAD_CHARS, THREAD_MAX } from './presence-thread.constants';

const presenceThread = (search: string): string => (new URLSearchParams(search).get('thread') ?? '').replace(THREAD_CHARS, '').slice(0, THREAD_MAX);

export { presenceThread };
