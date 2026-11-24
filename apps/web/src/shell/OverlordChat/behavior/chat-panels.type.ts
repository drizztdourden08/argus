/* @layer renderer-app @kind types */
import type { ReactNode } from 'react';
import type { ActionState, OverlordQueue, OverlordRule, RuleDraft } from '../../../data/overlord';

type Editing = OverlordRule | 'new' | null;

interface QueueFeed {
  queue?: OverlordQueue | null;
  problem?: string | null;
  action: ActionState;
  reload: () => void;
}

interface QueueFeedProps {
  feed: QueueFeed;
}

type ChatTab = 'chat' | 'rules' | 'queue' | 'settings';

type QueueSectionName = 'wakes' | 'held' | 'alerts';

interface OverlordRuleCardProps {
  rule: OverlordRule;
  busy: boolean;
  onToggle: (enabled: boolean) => void;
  onEdit: () => void;
  onRun: () => void;
  onDelete: () => void;
}

interface RuleFormProps {
  rule: OverlordRule | null;
  onClose: () => void;
}

interface RuleFormState {
  draft: RuleDraft;
  action: ActionState;
  change: (patch: Partial<RuleDraft>) => void;
  save: () => void;
}

interface QueueSectionProps {
  title: string;
  count: number;
  children: ReactNode;
}

interface QueueLineProps {
  text: string | null;
  label?: string | null;
  at?: string;
}

interface ChatOpen {
  open: boolean;
  toggle: () => void;
  close: () => void;
}

export type { ChatOpen, OverlordRuleCardProps, ChatTab, Editing, QueueFeedProps, QueueLineProps, QueueSectionName, QueueSectionProps, RuleFormProps, RuleFormState };
