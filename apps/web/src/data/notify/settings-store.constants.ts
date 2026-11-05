/* @layer renderer-app @kind constants */
import type { NotifyState } from './notify.type';

const EMPTY_AUTO = { handed: false, ask: false, assigned: false, verify: false, run: false };

const EMPTY: NotifyState = { toast: {}, sound: {}, os: false, auto: EMPTY_AUTO, ready: false };

export { EMPTY, EMPTY_AUTO };
