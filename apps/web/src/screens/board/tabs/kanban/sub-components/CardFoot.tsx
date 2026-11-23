/* @layer renderer-app @kind component */
import { Flex, Spacer, Tag, Text } from '@drizztdourden08/tessera';
import { formatAgo } from '../../../../../format';
import type { CardFootProps } from '../kanban-props.type';
import { cardChips, cardProgress, stageLabel } from '../model';
import { CardBars } from './CardBars';
import { CardProgressRow } from './CardProgressRow';

const CardFoot = (props: CardFootProps) => {
  const { card, group, now } = props;
  const { item } = card;
  const stage = stageLabel(item, group);
  return (
    <Flex direction="column" gap="xs">
      <Flex gap="sm" align="center">
        {cardChips(item).map((chip) => <Tag key={chip.id} title={chip.title}>{chip.text}</Tag>)}
        {stage && <Text variant="caption" tone="muted" title="Stage">{stage}</Text>}
        <CardProgressRow segments={cardProgress(card)} />
        <Spacer />
        <Text variant="caption" tone="muted">{formatAgo(item.updatedAt, now)}</Text>
      </Flex>
      <CardBars card={card} />
    </Flex>
  );
};

export { CardFoot };
