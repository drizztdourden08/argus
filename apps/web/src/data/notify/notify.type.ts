/* @layer renderer-app @kind types */
import type { LevelSwitches } from '../types';

interface SettingsPatch {
  toast?: LevelSwitches;
  sound?: LevelSwitches;
}

export type { SettingsPatch };
