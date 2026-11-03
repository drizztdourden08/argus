/* @layer renderer-app @kind types */
import type { BoardAction, Entry, LogLine, WorkItem } from '../../../data/types';

interface Proof {
  n: number;
  url: string;
  caption?: string;
}

interface ManualTest {
  id: string;
  title: string;
  thread: string;
  mode: string;
  status: string;
  updatedAt: string;
  item?: string | null;
  build?: string | null;
  rank?: number;
  proofs?: Proof[];
  archivedAt?: string;
  deletedAt?: string;
}

interface BuildRef {
  id: string;
  closedAt?: string | null;
}

interface BundleGroup {
  id: string;
  build: string;
  thread: string;
  tests: ManualTest[];
}

type FeedKind = 'decision' | 'question' | 'message' | 'notice' | 'action' | 'test' | 'bundle' | 'sent';

type AskKind = 'decision' | 'question' | 'message' | 'notice';

type FeedCard =
  | { type: AskKind | 'sent'; at: string; data: Entry }
  | { type: 'action'; at: string; data: BoardAction }
  | { type: 'test'; at: string; data: ManualTest }
  | { type: 'bundle'; at: string; data: BundleGroup };

type AskMode = 'decide' | 'answer' | 'info';

interface OwnerMarks {
  acked: ReadonlySet<string>;
  answered: ReadonlySet<string>;
  done: ReadonlySet<string>;
}

interface FeedInput {
  entries: Entry[];
  actions: BoardAction[];
  tests: ManualTest[];
  closed: ReadonlySet<string>;
  marks: OwnerMarks;
}

interface WorkSources {
  entries: Entry[];
  actions: BoardAction[];
  tests: ManualTest[];
  builds: BuildRef[];
  log: { subject?: string; entries?: LogLine[] }[];
  items: WorkItem[];
}

interface NeedCount {
  questions: number;
  decisions: number;
  steps: number;
  tests: number;
}

type ItemNeeds = ReadonlyMap<string, NeedCount>;

type FeedSection = 'waiting' | 'leads' | 'log';

interface BoardWork {
  ready: boolean;
  waiting: FeedCard[];
  leads: FeedCard[];
  notices: FeedCard[];
  lines: LogLine[];
  items: WorkItem[];
  index: ReadonlyMap<string, WorkItem>;
  needs: ItemNeeds;
}

interface ContextParts {
  shown: string;
  rest: string;
}

export type {
  AskKind,
  AskMode,
  BoardWork,
  BuildRef,
  BundleGroup,
  ContextParts,
  FeedCard,
  FeedInput,
  FeedKind,
  FeedSection,
  ItemNeeds,
  ManualTest,
  NeedCount,
  OwnerMarks,
  WorkSources,
};
