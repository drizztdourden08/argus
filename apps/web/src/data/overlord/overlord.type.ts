/* @layer renderer-app @kind types */
interface KeyStatus {
  set: boolean;
  last4: string | null;
  savedAt: string | null;
}

interface OverlordKeyApi {
  status: KeyStatus | null;
  busy: boolean;
  problem: string | null;
  save: (key: string) => Promise<boolean>;
  remove: () => Promise<boolean>;
}

type OverlordState = 'stopped' | 'starting' | 'idle' | 'turn' | 'paused-budget' | 'waiting-approval' | 'crashed';

type OverlordMode = 'off' | 'shadow' | 'live';

interface ModeSwitch {
  from: OverlordMode;
  to: OverlordMode;
  already: boolean;
}

type OverlordAction = 'start' | 'stop' | 'restart';

interface OverlordStatus {
  mode: string | null;
  status: OverlordState;
  pid: number | null;
  startedAt: string | null;
  sessionId: string | null;
}

interface OverlordSpend {
  month: string;
  spentUsd: number;
  budgetUsd: number;
  warnAt: number;
  leftUsd: number;
  note: string;
}

type ApprovalState = 'pending' | 'allowed' | 'denied' | 'expired';

type ApprovalDecision = 'allow' | 'deny';

interface OverlordApproval {
  id: string;
  at: string;
  tool: string;
  inputSummary: string;
  status: ApprovalState;
}

interface TranscriptEvent {
  type?: string;
  message?: { content?: unknown };
}

interface TranscriptLine {
  index: number;
  event: TranscriptEvent | string;
}

interface TranscriptPage {
  session: string | null;
  lines: (TranscriptEvent | string)[];
  next: number;
}

interface ActionState {
  busy: boolean;
  problem: string | null;
  run: (action: () => Promise<unknown>) => Promise<boolean>;
}

interface TranscriptFeed {
  lines: TranscriptLine[];
  problem: string | null;
}

export type {
  ActionState,
  ApprovalDecision,
  KeyStatus,
  ModeSwitch,
  OverlordAction,
  OverlordApproval,
  OverlordKeyApi,
  OverlordMode,
  OverlordSpend,
  OverlordState,
  OverlordStatus,
  TranscriptEvent,
  TranscriptFeed,
  TranscriptLine,
  TranscriptPage,
};
