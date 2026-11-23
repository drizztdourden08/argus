/* @layer renderer-app @kind types */
interface OverlordChoice {
  id: string;
  label: string;
}

type LimitSource = 'set' | 'board' | 'default';

interface LimitEntry<T> {
  value: T;
  source: LimitSource;
}

interface OverlordSettings {
  model: string;
  effort: string;
  permissionMode: string;
  choices: { models: OverlordChoice[]; efforts: string[]; permissionModes: OverlordChoice[] };
  cwd: LimitEntry<string>;
  exePath: LimitEntry<string>;
  budgetUsd: LimitEntry<number | null>;
  restartMax: LimitEntry<number>;
  approvalTimeoutMin: LimitEntry<number>;
}

type LimitKey = 'cwd' | 'exePath' | 'budgetUsd' | 'restartMax' | 'approvalTimeoutMin';

interface OverlordSettingsPatch {
  model?: string;
  effort?: string;
  permissionMode?: string;
  cwd?: string;
  exePath?: string;
  budgetUsd?: number | '';
  restartMax?: number | '';
  approvalTimeoutMin?: number | '';
}

interface SettingsDraft {
  cwd: string;
  exePath: string;
  budgetUsd: number;
  restartMax: number;
  approvalTimeoutMin: number;
}

export type { LimitKey, LimitSource, OverlordSettings, OverlordSettingsPatch, SettingsDraft };
