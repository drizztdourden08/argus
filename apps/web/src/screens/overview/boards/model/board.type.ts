/* @layer renderer-app @kind types */
import type { BoardItem } from '../../../../data/boards';
import type { WithId } from '../../../../data/react';
import type { RunTone } from '../../../../data/runs';
import type { RegistryFault, SentEntry } from '../../../../data/types';

type LeadState = 'not started' | 'paused' | 'idle' | 'stale' | 'running';

interface Attention {
  needs: number;
  failing: number;
  blocked: number;
  decisions: number;
}

interface SendOutcome {
  ok: boolean;
  message: string;
}

type SendKind = 'decision' | 'followup';

type SentItem = WithId<SentEntry>;

interface MasterLine {
  head: string;
  at: string | null;
  tail: string;
  tone: RunTone;
  running: boolean;
}

type BoardItemDoc = BoardItem['doc'];

type BoardItemCard = BoardItem['card'];

interface BoardCounts {
  threads: number;
  working: number;
  blocked: number;
  needsYou: number;
  itemsOpen: number;
  doing: number;
  review: number;
  done: number;
  testsPass: number;
  tests: number;
  testsFailing: number;
  agents: number;
  paused: number;
}

interface BoardRecord {
  id: string;
  key: string;
  name: string;
  icon: string | null;
  palette: string | null;
  goal: string;
  note: string;
  rev: number | undefined;
  archived: boolean;
  started: boolean;
  lead: LeadState;
  masterRunning: boolean;
  hasMaster: boolean;
  masterMissing: boolean;
  decisions: number;
  counts: BoardCounts;
}

interface BoardLive {
  fault: RegistryFault | null;
  stalls: number;
  stallLines?: readonly string[];
  syncDue: boolean;
}

type SignalId = 'needs-you' | 'running' | 'failing' | 'blocked' | 'stalled' | 'fault' | 'sync-due' | 'master-missing' | 'paused' | 'archived';

type SignalTone = 'bad' | 'warn' | 'good' | 'neutral' | 'muted';

type SignalTab = 'overview' | 'work' | 'tests' | 'activity' | 'builds';

interface BoardSignal {
  id: SignalId;
  tone: SignalTone;
  count: number;
  label: string;
  tab: SignalTab;
}

interface BoardProgress {
  done: number;
  review: number;
  doing: number;
  todo: number;
  total: number;
  ratio: number;
}

type BoardActionId = 'open' | 'send' | 'play' | 'stop' | 'edit' | 'archive' | 'unarchive';

interface BoardMenuItem {
  id: BoardActionId;
  label: string;
  tone?: 'danger';
}

type AttentionGroupId = 'needs-you' | 'running' | 'quiet' | 'not-started' | 'archived';

interface BoardGroup {
  id: AttentionGroupId;
  label: string;
  folded: boolean;
  boards: BoardRecord[];
}

export type {
  Attention, AttentionGroupId, BoardActionId, BoardCounts, BoardGroup, BoardItemCard, BoardItemDoc, BoardLive, BoardMenuItem, BoardProgress, BoardRecord, BoardSignal, LeadState, MasterLine, SendKind,
  SendOutcome, SentItem, SignalId, SignalTone,
};
