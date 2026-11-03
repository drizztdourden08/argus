/* @layer renderer-app @kind types */
import type { WithId } from '../../../../data/react';
import type { NotificationDoc, NotifyLevel } from '../../../../data/types';
import type { SETTING_PARTS } from './model.constants';

type NoteItem = WithId<NotificationDoc>;

type LevelPick = 'all' | NotifyLevel;

type SettingPart = (typeof SETTING_PARTS)[number];

interface NoteFilters {
  level: LevelPick;
  project: string;
}

interface OwnerRow {
  project: string;
  name: string;
  id: string;
  title: string;
  type: string;
  stage: string;
}

interface NoteParts {
  title: string;
  detail: string;
}

interface ToastNote {
  text: string;
  bad: boolean;
}

export type { LevelPick, NoteFilters, NoteItem, NoteParts, OwnerRow, SettingPart, ToastNote };
