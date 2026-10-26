/* @layer renderer-app @kind types */
import type { BoardAction, Entry, LogLine } from '../../../data/types';
import type { AskMode, BoardWork, FeedCard, Proof } from '../feed-model';
import type { NoteKind } from '../owner-posts';

type FeedFilter = 'all' | 'you' | 'log';

interface FeedActions {
  isBusy: (key: string) => boolean;
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
  id: string;
  at: string;
  thread?: string | null;
  item?: string | null;
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

interface TestBodyProps {
  summary: string;
  title: string;
  build?: string | null;
  proofs?: Proof[];
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
  TestBodyProps,
};
