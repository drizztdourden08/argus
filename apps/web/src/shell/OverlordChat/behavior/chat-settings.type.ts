/* @layer renderer-app @kind types */
import type { ReactNode } from 'react';
import type { ActionState, LimitKey, LimitSource, OverlordSettings, OverlordUsage, SettingsDraft } from '../../../data/overlord';

interface SettingsFormState {
  settings: OverlordSettings | null;
  draft: SettingsDraft | null;
  action: ActionState;
  change: (patch: Partial<SettingsDraft>) => void;
  save: () => void;
  reset: (key: LimitKey) => void;
}

interface SettingsFieldProps {
  name: LimitKey;
  source: LimitSource;
  busy: boolean;
  onReset: (key: LimitKey) => void;
  children: ReactNode;
}

interface SettingChoice {
  id: string;
  label: string;
}

type SettingKey = 'model' | 'effort' | 'permissionMode';

interface SettingChipProps {
  label: string;
  current: string;
  selected: string;
  choices: SettingChoice[];
  busy: boolean;
  onPick: (id: string) => void;
}

interface ChatSettings {
  settings: OverlordSettings | null;
  busy: boolean;
  problem: string | null;
  pick: (key: SettingKey, id: string) => void;
}

interface ChatUsage {
  usage: OverlordUsage | null;
  reload: () => void;
}

interface ChatBarProps {
  working: boolean;
}

interface UsageRingProps {
  usage: ChatUsage;
}

interface UsagePanelProps {
  usage: OverlordUsage;
}

interface UsageRow {
  label: string;
  value: string;
}

interface UsageSection {
  title: string;
  rows: UsageRow[];
}

interface ChipSpec {
  key: SettingKey;
  label: string;
  current: string;
  selected: string;
  choices: SettingChoice[];
}

interface SettingChipsProps {
  chat: ChatSettings;
}

export type { ChatBarProps, ChatSettings, ChatUsage, ChipSpec, SettingChipProps, SettingChipsProps, SettingChoice, SettingKey, SettingsFieldProps, SettingsFormState, UsagePanelProps, UsageRingProps, UsageRow, UsageSection };
