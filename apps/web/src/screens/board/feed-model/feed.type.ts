/* @layer renderer-app @kind types */
import type { BoardAction, Entry, WorkItem } from '../../../data/types';

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

interface RunCard {
  id: string;
  n: number;
  plan: string;
  planTitle: string;
  build: string | null;
  cases: number;
  thread: string;
  reported: boolean;
}

type FeedKind = 'decision' | 'question' | 'message' | 'action' | 'test' | 'run';

type AskKind = 'decision' | 'question' | 'message';

type FeedCard =
  | { type: AskKind; at: string; data: Entry }
  | { type: 'action'; at: string; data: BoardAction }
  | { type: 'test'; at: string; data: ManualTest }
  | { type: 'run'; at: string; data: RunCard };

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
  items: WorkItem[];
  marks: OwnerMarks;
}

interface WorkSources {
  entries: Entry[];
  actions: BoardAction[];
  tests: ManualTest[];
  items: WorkItem[];
}

interface NeedCount {
  questions: number;
  decisions: number;
  steps: number;
  tests: number;
}

type ItemNeeds = ReadonlyMap<string, NeedCount>;

interface BoardWork {
  ready: boolean;
  asks: FeedCard[];
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
  ContextParts,
  FeedCard,
  FeedInput,
  FeedKind,
  ItemNeeds,
  ManualTest,
  NeedCount,
  OwnerMarks,
  RunCard,
  WorkSources,
};
