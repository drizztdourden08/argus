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
  claudeExe: LimitEntry<string>;
  budgetUsd: LimitEntry<number | null>;
  restartMax: LimitEntry<number>;
  approvalTimeoutMin: LimitEntry<number>;
}

type LimitKey = 'cwd' | 'claudeExe' | 'budgetUsd' | 'restartMax' | 'approvalTimeoutMin';

/** An empty text or an empty string in place of a number resets the field to its default. */
interface OverlordSettingsPatch {
  model?: string;
  effort?: string;
  permissionMode?: string;
  cwd?: string;
  claudeExe?: string;
  budgetUsd?: number | '';
  restartMax?: number | '';
  approvalTimeoutMin?: number | '';
}

/** The settings form; a number that is not finite is a blank field. */
interface SettingsDraft {
  cwd: string;
  claudeExe: string;
  budgetUsd: number;
  restartMax: number;
  approvalTimeoutMin: number;
}

export type { LimitEntry, LimitKey, LimitSource, OverlordSettings, OverlordSettingsPatch, SettingsDraft };
