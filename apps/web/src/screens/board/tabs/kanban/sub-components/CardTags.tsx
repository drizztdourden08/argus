/* @layer renderer-app @kind component */
import { Flex, Tag } from '@drizztdourden08/tessera';
import { formatAgo } from '../../../../../format';
import type { CardTagsProps } from '../kanban-props.type';
import { quietTip } from '../model';

const CardTags = (props: CardTagsProps) => {
  const { card, group, now } = props;
  const { item } = card;
  return (
    <Flex gap="xs" align="center" wrap>
      {card.needsYou && <Tag variant="urgency" color="warning" title="A question, a to-do or a test waits on you.">needs you</Tag>}
      {card.decisions > 0 && <Tag variant="urgency" color="info" title="Decisions to pick up">{`${card.decisions} to decide`}</Tag>}
      {card.quietMs !== null && <Tag variant="urgency" color="warning" title={quietTip(item, now)}>{`quiet ${formatAgo(item.updatedAt, now)}`}</Tag>}
      {group !== 'status' && <Tag>{item.status}</Tag>}
      {card.conflicted && <Tag variant="urgency" color="danger">conflict</Tag>}
      {item.archivedAt && <Tag>archived</Tag>}
    </Flex>
  );
};

export { CardTags };
