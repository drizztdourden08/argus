/* @layer renderer-app @kind types */
import type { ReactNode } from 'react';
import type { BoardAction, Entry, EntryImage, LogLine, WorkItem } from '../../../data/types';
import type { ThreadChoice } from '../../../data/kanban';
import type { WithId } from '../../../data/react';
import type { AskMode, BoardWork, FeedCard, ManualTest, RunCard } from '../feed-model';
import type { NoteKind } from '../owner-posts';

type AskFeedCard = Extract<FeedCard, { data: Entry }>;

interface AskLines {
  title: string;
  rest: string;
}

type FeedFilter = 'all' | 'you' | 'log';

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
  tab?: string;
}

type TestRef = Pick<ManualTest, 'id' | 'thread' | 'item'>;

interface LogWindow {
  shown: number;
  left: number;
  more: () => void;
  less: () => void;
}

interface OwnerFeedProps {
  boardKey: string;
  work: BoardWork;
}

interface FeedGroupProps {
  heading: string;
  cards: FeedCard[];
  ctx: FeedContext;
  total?: number;
  extra?: ReactNode;
}

interface AskGroupProps {
  heading: string;
  cards: AskFeedCard[];
  ctx: FeedContext;
}

interface LeadNotesGroup {
  lead: string;
  entries: Entry[];
}

interface LeadNotesProps {
  cards: FeedCard[];
  ctx: FeedContext;
}

interface SentRowProps {
  entry: Entry;
  ctx: FeedContext;
}

interface FeedCardViewProps {
  card: FeedCard;
  ctx: FeedContext;
}

interface AskCardProps {
  card: AskFeedCard;
  ctx: FeedContext;
}

interface FeedFactsProps {
  facts: CardFacts;
  index: ReadonlyMap<string, WorkItem>;
}

interface ItemChipProps {
  id: string;
  index: ReadonlyMap<string, WorkItem>;
}

interface CardMenuProps {
  collection: 'entries' | 'actions';
  id: string;
  actions: FeedActions;
}

interface BuildTagProps {
  build: string | null;
}

interface RunBodyProps {
  run: RunCard;
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

interface AskFormProps extends EntryBodyProps {
  mode: AskMode;
  quick?: boolean;
}

interface ContextPreviewProps extends EntryBodyProps {
  onMore: () => void;
}

interface AskCopy {
  kind: NoteKind;
  label: string;
  verb: string;
  fold: string;
}

interface ActionBodyProps {
  action: BoardAction;
  actions: FeedActions;
}

interface NoticeRowProps {
  entry: Entry;
  ctx: FeedContext;
}

interface NoticeListProps {
  notices: FeedCard[];
  ctx: FeedContext;
}

interface LogListProps {
  lines: LogLine[];
}

interface LogSectionProps {
  lines: LogLine[];
}

interface LogFoldProps {
  filter: FeedFilter;
  notices: FeedCard[];
  lines: LogLine[];
  ctx: FeedContext;
}

interface FeedSectionsProps {
  filter: FeedFilter;
  work: BoardWork;
  ctx: FeedContext;
}

export type {
  ActionBodyProps,
  AskCardProps,
  AskCopy,
  AskGroupProps,
  AskFeedCard,
  AskLines,
  AskFormProps,
  BuildTagProps,
  CardFacts,
  CardMenuProps,
  ContextPreviewProps,
  EntryBodyProps,
  EntryImagesProps,
  FeedActions,
  FeedCardViewProps,
  FeedContext,
  FeedFactsProps,
  FeedFilter,
  FeedFilters,
  FeedGroupProps,
  FeedSectionsProps,
  ImageGroup,
  ItemChipProps,
  LeadNotesGroup,
  LeadNotesProps,
  LogFoldProps,
  LogListProps,
  LogSectionProps,
  LogWindow,
  NoticeListProps,
  NoticeRowProps,
  OwnerFeedProps,
  SentRowProps,
  RunBodyProps,
  TestRef,
};
