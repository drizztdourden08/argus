/* @layer renderer-app @kind component */
import { Tag } from '@drizztdourden08/tessera';
import { formatAgo } from '../../../../../format';
import { DepTag } from '../../../../../shell';
import type { CardTagsProps } from '../kanban-props.type';
import { NEEDS_YOU_TIP } from '../kanban.constants';
import { depTags, quietTip } from '../model';
import { AskTag } from './AskTag';
import { StatusTag } from './StatusTag';

const CardTags = (props: CardTagsProps) => {
  const { card, group, now } = props;
  const { item } = card;
  const blockerLinked = card.asks.some((link) => link.via === 'blocker');
  return (
    <>
      {card.needsYou && <Tag variant="urgency" color="danger" title={NEEDS_YOU_TIP}>needs you</Tag>}
      {card.asks.map((link) => <AskTag key={link.id} link={link} />)}
      {card.quietMs !== null && <Tag variant="urgency" color="warning" title={quietTip(item, now)}>{`quiet ${formatAgo(item.updatedAt, now)}`}</Tag>}
      {group !== 'status' && <StatusTag status={item.status} />}
      {depTags(item, now).map((tag) => <DepTag key={tag.key} color={tag.color} title={tag.title}>{tag.label}</DepTag>)}
      {item.blocker && !blockerLinked && <Tag variant="urgency" color="warning" title={item.blocker}>{`blocked: ${item.blocker}`}</Tag>}
      {card.conflicted &&<Tag variant="urgency" color="danger">conflict</Tag>}
      {item.archivedAt && <Tag>archived</Tag>}
    </>
  );
};

export { CardTags };
