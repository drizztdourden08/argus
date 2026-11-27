/* @layer renderer-app @kind logic */
import type { SwitchThreadDoc } from '../../../../../data/types';
import { PENDING_STATES } from './is-thread-pending.constants';

const isThreadPending = (thread: SwitchThreadDoc): boolean => PENDING_STATES.includes(thread.state);

export { isThreadPending };
