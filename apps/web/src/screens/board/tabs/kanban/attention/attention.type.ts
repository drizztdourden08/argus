/* @layer renderer-app @kind types */
import type { IconName, TagUrgencyColor } from '@drizztdourden08/tessera';
import type { ItemRef } from '../../../../../data/kanban';
import type { AskLink, Needs } from '../model/model.type';

type AttentionKind = 'question' | 'todo' | 'test-failing' | 'dependency' | 'blocked' | 'test-pending' | 'decision' | 'quiet' | 'agent';

interface AttentionSpec {
  kind: AttentionKind;
  icon: IconName;
  color: TagUrgencyColor;
  label: string;
  text: string;
}

interface Signals {
  failing: number;
  pending: number;
  agents: number;
}

interface AttentionFacts {
  item: ItemRef;
  needs: Needs | undefined;
  asks: readonly AskLink[];
  signals: Signals;
  quietMs: number | null;
}

interface SignalsInput {
  tests: readonly { item: string | null; status: string; archivedAt?: string | null; deletedAt?: string | null; build?: string | null }[];
  agents: readonly { item?: string | null; status?: string; archivedAt?: string | null; deletedAt?: string | null }[];
  closedBuilds: ReadonlySet<string>;
}

export type { AttentionFacts, AttentionKind, AttentionSpec, Signals, SignalsInput };
