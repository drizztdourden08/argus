/* @layer renderer-app @kind types */
import type { LevelSwitches, NotifyLevel, TestPingResult } from '../types';

interface SettingsPatch {
  toast?: LevelSwitches;
  sound?: LevelSwitches;
}

interface AutoLevels {
  handed: boolean;
  ask: boolean;
  assigned: boolean;
  verify: boolean;
  run: boolean;
}

interface AutoAnswer {
  os: boolean;
  auto: AutoLevels;
}

interface AutoPatch {
  os?: boolean;
  auto?: Partial<AutoLevels>;
}

interface LevelPatch {
  toast?: boolean;
  sound?: boolean;
}

interface NotifyState {
  toast: LevelSwitches;
  sound: LevelSwitches;
  os: boolean;
  auto: AutoLevels;
  ready: boolean;
}

type Refusal = string | null;

interface NotifySettingsHook extends NotifyState {
  saveLevel: (level: NotifyLevel, patch: LevelPatch) => Promise<Refusal>;
  saveAuto: (patch: AutoPatch) => Promise<Refusal>;
  sendTest: () => Promise<TestPingResult>;
}

export type { AutoAnswer, AutoLevels, AutoPatch, LevelPatch, NotifySettingsHook, NotifyState, Refusal, SettingsPatch };
