/* @layer renderer-app @kind component */
import { Flex, Text } from '@drizztdourden08/tessera';
import type { AskGroupProps } from '../OwnerFeed.type';
import { AskCard } from './AskCard';

const AskGroup = (props: AskGroupProps) => {
  const { heading, cards, ctx } = props;
  return (
    <Flex direction="column" gap="sm" role="group" aria-label={heading}>
      <Text variant="overline" tone="muted">{heading} · {cards.length}</Text>
      {cards.map((card) => (
        <AskCard key={`${card.type}:${card.data.id}`} card={card} ctx={ctx} />
      ))}
    </Flex>
  );
};

export { AskGroup };
