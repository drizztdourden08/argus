/* @layer renderer-app @kind types */
import type { WithId } from '../../../../data/react';
import type { NotificationDoc, NotifyLevel } from '../../../../data/types';

type NoteItem = WithId<NotificationDoc>;

type LevelPick = 'all' | NotifyLevel;

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

export type { LevelPick, NoteFilters, NoteItem, NoteParts, OwnerRow, ToastNote };
