/* @layer renderer-app @kind types */
import type { BoardAction, Entry, EntryImage, WorkItem } from '../../../data/types';
import type { ThreadChoice } from '../../../data/kanban';
import type { WithId } from '../../../data/react';
import type { BoardWork } from '../feed-model';
import type { NoteKind } from '../owner-posts';

interface FeedFilters {
  query: string;
  authors: string[];
  kinds: string[];
}

interface FeedActions {
  isBusy: (key: string) => boolean;
  soft: (kind: 'archive' | 'delete', c: 'entries' | 'actions', id: string) => Promise<boolean>;
  choose: (entry: Entry, option: string) => Promise<boolean>;
  reply: (entry: Entry, kind: NoteKind, text: string) => Promise<boolean>;
  dismiss: (entry: Entry) => Promise<boolean>;
  done: (action: BoardAction) => Promise<boolean>;
  askContext: (entry: Entry) => Promise<boolean>;
}

interface FeedContext {
  boardKey: string;
  threads: readonly WithId<ThreadChoice>[];
  now: number;
  index: ReadonlyMap<string, WorkItem>;
  actions: FeedActions;
}

interface CardFacts {
  kind: string;
  code: string;
  title: string;
  thread: string | null;
  item: string | null;
  at: string;
  assignee: string;
}

interface OwnerFeedProps {
  boardKey: string;
  work: BoardWork;
}

interface CardMenuProps {
  collection: 'entries' | 'actions';
  id: string;
  actions: FeedActions;
}

interface EntryBodyProps {
  entry: Entry;
  actions: FeedActions;
}

interface EntryImagesProps {
  entry: Entry;
}

interface ImageGroup {
  key: string;
  title: string | null;
  shots: EntryImage[];
}

export type {
  CardFacts,
  CardMenuProps,
  EntryBodyProps,
  EntryImagesProps,
  FeedActions,
  FeedContext,
  FeedFilters,
  ImageGroup,
  OwnerFeedProps,
};
