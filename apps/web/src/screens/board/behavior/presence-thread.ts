/* @layer renderer-app @kind logic */
import { canonicalThread } from '../../../data/content';
import { THREAD_CHARS, THREAD_MAX } from './presence-thread.constants';

const presenceThread = (search: string): string => canonicalThread((new URLSearchParams(search).get('thread') ?? '').replace(THREAD_CHARS, '').slice(0, THREAD_MAX));

export { presenceThread };
