/* @layer renderer-app @kind component */
import { Tag } from '@drizztdourden08/tessera';
import type { CardTagsProps } from '../kanban-props.type';
import { NEEDS_YOU_TIP } from '../kanban.constants';
import { ATTENTION_OF } from '../attention';
import { AttentionTag } from './AttentionTag';
import { DepTagList } from './DepTagList';
import { StatusTag } from './StatusTag';

const needsIcon = (card: CardTagsProps['card']) => {
  const lead = card.attention.find((kind) => kind === 'question' || kind === 'todo') ?? 'test-pending';
  return ATTENTION_OF[lead].icon;
};

const CardTags = (props: CardTagsProps) => {
  const { card, group, now } = props;
  const { item } = card;
  return (
    <>
      {card.needsYou && <Tag variant="urgency" color="danger" icon={needsIcon(card)} title={NEEDS_YOU_TIP}>needs you</Tag>}
      {card.attention.map((kind) => <AttentionTag key={kind} kind={kind} card={card} now={now} />)}
      {group !== 'status' && <StatusTag status={item.status} />}
      {!card.attention.includes('dependency') && <DepTagList item={item} now={now} />}
      {card.conflicted && <Tag variant="urgency" color="danger" icon="triangle-alert">conflict</Tag>}
      {item.archivedAt && <Tag>archived</Tag>}
    </>
  );
};

export { CardTags };
