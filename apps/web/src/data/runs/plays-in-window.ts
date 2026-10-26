/* @layer renderer-app @kind logic */
import type { LaunchDef } from '../types';

const playsInWindow = (def: LaunchDef): boolean => {
  if (!def.hand) return false;
  return def.launcher || def.bundle ? def.window === true : def.visible === true;
};

export { playsInWindow };
