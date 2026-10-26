/* @layer renderer-app @kind logic */
import type { TagLook } from '@drizztdourden08/tessera';
import { KIND_LOOK, QUIET_LOOK } from './model.constants';

const kindLook = (kind: string): TagLook => KIND_LOOK[kind] ?? QUIET_LOOK;

export { kindLook };
