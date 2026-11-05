/* @layer renderer-app @kind types */
import type { LevelPick, NoteFilters, NoteItem } from './model';

interface NotificationsTabProps {
  freshSince: number;
}

interface TestPing {
  busy: boolean;
  message: string;
  send: () => void;
}

interface TestPingProps {
  ping: TestPing;
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

export type { NoteFeedProps, NoteFiltersProps, NoteRowProps, NotificationsTabProps, TestPing, TestPingProps };
