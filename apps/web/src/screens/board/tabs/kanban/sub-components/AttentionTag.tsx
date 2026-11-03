/* @layer renderer-app @kind component */
import { Tag } from '@drizztdourden08/tessera';
import { ATTENTION_OF } from '../attention';
import type { AttentionTagProps } from '../kanban-props.type';
import { attentionText } from '../model';
import { AskTag } from './AskTag';
import { DepTagList } from './DepTagList';

const AttentionTag = ({ kind, card, now }: AttentionTagProps) => {
  if (kind === 'question' || kind === 'decision') {
    return <>{card.asks.filter((link) => link.kind === kind).map((link) => <AskTag key={link.id} link={link} />)}</>;
  }
  if (kind === 'dependency') return <DepTagList item={card.item} now={now} />;
  const text = attentionText(kind, card, now);
  if (!text) return null;
  const { color, icon } = ATTENTION_OF[kind];
  return <Tag variant="urgency" color={color} icon={icon} title={text.title}>{text.label}</Tag>;
};

export { AttentionTag };
