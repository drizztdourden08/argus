/* @layer renderer-app @kind types */
import type { ActionDoc } from '../../../../../data/kanban';
import type { WithId } from '../../../../../data/react';
import type { Entry as BoardEntry, LogDoc } from '../../../../../data/types';
import type { OwnerMarks } from '../../../feed-model';

interface EntryRow {
  entry: BoardEntry;
  open: boolean;
  replies: BoardEntry[];
}

interface EntryState {
  entries: BoardEntry[];
  marks: OwnerMarks;
  opened: Set<string>;
}

interface TodoRow {
  action: WithId<ActionDoc>;
  done: boolean;
}

interface TimelineRow {
  key: string;
  at: string;
  kind: string;
  who: string;
  id: string;
  text: string;
  status: string | null;
  on: string | null;
  doneable: boolean;
}

interface RecordInput {
  itemId: string;
  entries: readonly BoardEntry[];
  actions: readonly WithId<ActionDoc>[];
  log: readonly LogDoc[];
}

interface TimelineInput {
  itemId: string;
  entries: readonly BoardEntry[];
  opened: ReadonlySet<string>;
  todos: readonly TodoRow[];
  log: readonly LogDoc[];
}

interface ItemRecord {
  questions: EntryRow[];
  decisions: EntryRow[];
  todos: TodoRow[];
  timeline: TimelineRow[];
}

export type { EntryRow, EntryState, ItemRecord, RecordInput, TimelineInput, TimelineRow, TodoRow };
