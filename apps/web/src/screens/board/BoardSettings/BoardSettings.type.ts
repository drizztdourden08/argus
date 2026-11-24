/* @layer renderer-app @kind types */
import type { IconPickerValue } from '@drizztdourden08/tessera';
import type { ReactNode } from 'react';
import type { BoardIconName } from '../../../board-icon';
import type { PostResult } from '../../../data/api';

type SettingsTab = 'appearance' | 'general' | 'kanban';

interface BoardSettingsButtonProps {
  boardKey: string;
}

interface BoardSettingsDialogProps {
  boardKey: string;
  onClose: () => void;
}

interface SectionProps {
  boardKey: string;
}

interface SettingSave {
  busy: boolean;
  problem: string | null;
  setProblem: (problem: string | null) => void;
  save: (value: string | null) => Promise<void>;
}

interface SettingSaveOptions {
  boardKey: string;
  post: (key: string, value: string | null) => Promise<PostResult>;
  saved: string;
}

interface AvatarChoice {
  setting: string | null;
  busy: boolean;
  problem: string | null;
  pickIcon: (name: BoardIconName) => Promise<void>;
  upload: (files: File[]) => Promise<void>;
  reject: () => void;
  reset: () => Promise<void>;
}

interface IconChoice extends AvatarChoice {
  avatar: string | null;
}

interface LookPickerProps {
  title: string;
  note: string;
  preview: ReactNode;
  pickerLabel: string;
  value: IconPickerValue;
  accept: string[];
  uploadLabel: string;
  choice: AvatarChoice;
}

interface TitleLimitChoice {
  value: number;
  busy: boolean;
  problem: string | null;
  changed: boolean;
  setValue: (value: number) => void;
  commit: () => Promise<void>;
}

export type { AvatarChoice, BoardSettingsButtonProps, BoardSettingsDialogProps, IconChoice, LookPickerProps, SectionProps, SettingSave, SettingSaveOptions, SettingsTab, TitleLimitChoice };
