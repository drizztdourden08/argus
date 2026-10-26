/* @layer renderer-app @kind component */
import { Checkbox, Flex, Pressable, Text } from '@drizztdourden08/tessera';
import type { ItemCardProps } from '../kanban-props.type';
import { CardFoot } from './CardFoot';
import { CardMenu } from './CardMenu';
import { CardTags } from './CardTags';

const ItemCard = (props: ItemCardProps) => {
  const { card, group, threads, now, selected, ops, onToggle, onOpen } = props;
  const { item } = card;
  return (
    <Flex direction="column" gap="xs" data-item={item.id}>
      <Flex gap="xs" align="center" wrap>
        <Checkbox checked={selected} onChange={() => onToggle(item.id)} ariaLabel={`Select ${item.id}`} size="sm" />
        <Text variant="caption" tone="muted" mono>{item.id}</Text>
        <CardTags card={card} group={group} now={now} />
        <CardMenu item={item} threads={threads} ops={ops} onOpen={onOpen} />
      </Flex>
      <Pressable onClick={() => onOpen(item.id)} aria-label={`Open ${item.id}: ${item.title}`} title={item.title}>
        <Text weight={500}>{item.title}</Text>
      </Pressable>
      <CardFoot card={card} group={group} now={now} />
    </Flex>
  );
};

export { ItemCard };
