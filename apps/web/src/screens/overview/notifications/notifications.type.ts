/* @layer renderer-app @kind types */
import type { NotifyLevel, NotifySettings } from '../../../data/types';
import type { LevelPick, NoteFilters, NoteItem, SettingPart } from './model';

interface NotificationsTabProps {
  freshSince: number;
}

interface SettingsState {
  settings: NotifySettings | null;
  error: string;
  busy: boolean;
  toggle: (part: SettingPart, level: NotifyLevel) => void;
}

interface TestPing {
  busy: boolean;
  message: string;
  send: () => void;
}

interface TestPingProps {
  ping: TestPing;
}

interface NotifySettingsProps {
  state: SettingsState;
}

interface SettingToggleProps {
  part: SettingPart;
  level: NotifyLevel;
  on: boolean;
  busy: boolean;
  onToggle: () => void;
}

interface NoteFiltersProps {
  filters: NoteFilters;
  counts: Record<LevelPick, number>;
  projects: readonly [string, number][];
  onChange: (next: NoteFilters) => void;
}

interface NoteFeedProps {
  notes: readonly NoteItem[];
  freshSince: number;
}

interface NoteRowProps {
  note: NoteItem;
  freshSince: number;
}

export type { NoteFeedProps, NoteFiltersProps, NoteRowProps, NotificationsTabProps, NotifySettingsProps, SettingsState, SettingToggleProps, TestPing, TestPingProps };
