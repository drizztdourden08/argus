/* @layer renderer-app @kind component */
import { Flex } from '@drizztdourden08/tessera';
import type { AskGroupProps } from '../behavior/ask.type';
import { AskCard } from './AskCard';

const AskGroup = (props: AskGroupProps) => {
  const { cards, ctx } = props;
  return (
    <Flex direction="column" gap="sm" role="list" aria-label="Questions and decisions">
      {cards.map((card) => (
        <AskCard key={`${card.type}:${card.data.id}`} card={card} ctx={ctx} />
      ))}
    </Flex>
  );
};

export { AskGroup };
