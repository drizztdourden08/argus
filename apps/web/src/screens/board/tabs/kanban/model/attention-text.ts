/* @layer renderer-app @kind logic */
import { formatAgo } from '../../../../../format';
import type { AttentionKind } from '../attention/attention.type';
import type { AttentionText, CardData } from './model.type';
import { quietTip } from './quiet-tip';

const agentLabel = (count: number): string => (count > 1 ? `${count} agents running` : 'agent running');

const attentionText = (kind: AttentionKind, card: CardData, now: number): AttentionText | null => {
  const { item, signals } = card;
  if (kind === 'test-failing') return { label: `${signals.failing} failing`, title: 'A test of this item failed, or is partial or blocked.' };
  if (kind === 'blocked') return { label: `blocked: ${item.blocker ?? ''}`, title: item.blocker ?? undefined };
  if (kind === 'test-pending') return { label: `${signals.pending} to run`, title: 'A test of this item has no result yet.' };
  if (kind === 'quiet') return { label: `quiet ${formatAgo(item.updatedAt, now)}`, title: quietTip(item, now) };
  if (kind === 'agent') return { label: agentLabel(signals.agents), title: 'A sub-agent is working on this item.' };
  return null;
};

export { attentionText };
