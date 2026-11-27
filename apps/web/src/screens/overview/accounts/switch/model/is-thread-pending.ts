/* @layer renderer-app @kind logic */
import type { SwitchThreadDoc } from '../../../../../data/types';

const PENDING_STATES: readonly (string | undefined)[] = [undefined, 'found', 'asked'];

const isThreadPending = (thread: SwitchThreadDoc): boolean => PENDING_STATES.includes(thread.state);

export { isThreadPending };
