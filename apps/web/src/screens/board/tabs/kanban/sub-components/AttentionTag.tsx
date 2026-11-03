/* @layer renderer-app @kind component */
import { Tag } from '@drizztdourden08/tessera';
import type { ReactNode } from 'react';
import { formatAgo } from '../../../../../format';
import { ATTENTION_OF } from '../attention';
import type { AttentionKind } from '../attention';
import type { AttentionTagProps } from '../kanban-props.type';
import { quietTip } from '../model';
import { AskTag } from './AskTag';
import { DepTagList } from './DepTagList';

const asksOf = (props: AttentionTagProps, kind: 'question' | 'decision'): ReactNode =>
  props.card.asks.filter((link) => link.kind === kind).map((link) => <AskTag key={link.id} link={link} />);

const tagOf = (kind: AttentionKind, children: ReactNode, title?: string): ReactNode => {
  const { color, icon } = ATTENTION_OF[kind];
  return <Tag variant="urgency" color={color} icon={icon} title={title}>{children}</Tag>;
};

const TAGS: Readonly<Record<AttentionKind, (props: AttentionTagProps) => ReactNode>> = {
  question: (props) => asksOf(props, 'question'),
  todo: () => null,
  'test-failing': ({ card }) => tagOf('test-failing', `${card.signals.failing} failing`, 'A test of this item failed, or is partial or blocked.'),
  dependency: ({ card, now }) => <DepTagList item={card.item} now={now} />,
  blocked: ({ card }) => tagOf('blocked', `blocked: ${card.item.blocker}`, card.item.blocker ?? undefined),
  'test-pending': ({ card }) => tagOf('test-pending', `${card.signals.pending} to run`, 'A test of this item has no result yet.'),
  decision: (props) => asksOf(props, 'decision'),
  quiet: ({ card, now }) => tagOf('quiet', `quiet ${formatAgo(card.item.updatedAt, now)}`, quietTip(card.item, now)),
  agent: ({ card }) => tagOf('agent', card.signals.agents > 1 ? `${card.signals.agents} agents running` : 'agent running', 'A sub-agent is working on this item.'),
};

const AttentionTag = (props: AttentionTagProps) => <>{TAGS[props.kind](props)}</>;

export { AttentionTag };
