/* @layer renderer-app @kind types */
import type { ReactNode } from 'react';
import type { BoardAction, Entry, LogLine } from '../../../data/types';
import type { AskMode, BoardWork, FeedCard } from '../feed-model';
import type { NoteKind } from '../owner-posts';

type FeedFilter = 'all' | 'you' | 'log';

interface FeedActions {
  isBusy: (key: string) => boolean;
  handed: string | null;
  soft: (kind: 'archive' | 'delete', c: 'entries' | 'actions', id: string) => Promise<boolean>;
  choose: (entry: Entry, option: string) => Promise<boolean>;
  reply: (entry: Entry, kind: NoteKind, text: string) => Promise<boolean>;
  dismiss: (entry: Entry) => Promise<boolean>;
  done: (action: BoardAction) => Promise<boolean>;
  askContext: (entry: Entry) => Promise<boolean>;
}

interface LogWindow {
  shown: number;
  left: number;
  more: () => void;
  less: () => void;
}

interface OwnerFeedProps {
  boardKey: string;
  work: BoardWork;
  handed: string | null;
}

interface FeedGroupProps {
  heading: string;
  cards: FeedCard[];
  actions: FeedActions;
}

interface FeedCardViewProps {
  card: FeedCard;
  actions: FeedActions;
}

interface CardMetaProps {
  label: string;
  id: string;
  at: string;
  thread?: string | null;
  item?: string | null;
  menu?: ReactNode;
}

interface CardMenuProps {
  collection: 'entries' | 'actions';
  id: string;
  actions: FeedActions;
}

interface TestLineProps {
  card: FeedCard & { type: 'test' | 'bundle' };
  handed: string | null;
}

interface TestLineView {
  build?: string | null;
  count: string;
  title: string;
  hover: string;
  thread: string;
  open: string;
}

interface EntryBodyProps {
  entry: Entry;
  actions: FeedActions;
}

interface AskFormProps extends EntryBodyProps {
  mode: AskMode;
}

interface AskCopy {
  kind: NoteKind;
  label: string;
  verb: string;
}

interface ActionBodyProps {
  action: BoardAction;
  actions: FeedActions;
}


interface LogListProps {
  lines: LogLine[];
}

interface LogSectionProps {
  lines: LogLine[];
}

interface CardSectionsProps {
  filter: FeedFilter;
  waiting: FeedCard[];
  notices: FeedCard[];
  actions: FeedActions;
}

interface FeedSectionsProps {
  filter: FeedFilter;
  work: BoardWork;
  actions: FeedActions;
}

export type {
  ActionBodyProps,
  AskCopy,
  AskFormProps,
  CardMenuProps,
  CardMetaProps,
  CardSectionsProps,
  EntryBodyProps,
  FeedActions,
  FeedCardViewProps,
  FeedFilter,
  FeedGroupProps,
  FeedSectionsProps,
  LogListProps,
  LogSectionProps,
  LogWindow,
  OwnerFeedProps,
  TestLineProps,
  TestLineView,
};
