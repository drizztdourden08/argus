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

type ModelColor = 'neutral' | 'primary' | 'secondary' | 'tertiary';

type AgentFilter = 'running' | 'done' | 'all';

interface Verdict {
  ok: boolean;
  message: string;
}

export type { AgentFilter, AgentMark, ModelColor, ModelFamily, NowLine, StatusLook, Verdict };
