/* @layer renderer-app @kind types */
import type { StatusTone } from '@drizztdourden08/tessera';

interface NowLine {
  text: string;
  since: string | null;
}

interface StatusLook {
  label: string;
  tone: StatusTone;
}

type MarkTone = 'warning' | 'danger' | 'info';

interface AgentMark {
  id: string;
  label: string;
  tone: MarkTone;
  tip: string;
}

type ModelFamily = 'haiku' | 'sonnet' | 'opus' | 'other';

interface ModelLook {
  tone: 'success' | 'info' | 'neutral';
  dot: boolean;
}

type AgentField = 'task' | 'note' | 'branch' | 'worktree';

type WhereKind = 'worktree' | 'master' | 'unknown';

interface WhereLook {
  kind: WhereKind;
  text: string;
  tip: string;
}

interface Verdict {
  ok: boolean;
  message: string;
}

export type { AgentField, AgentMark, ModelFamily, ModelLook, NowLine, StatusLook, Verdict, WhereLook };
