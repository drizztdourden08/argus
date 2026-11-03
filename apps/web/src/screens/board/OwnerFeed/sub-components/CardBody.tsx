/* @layer renderer-app @kind component */
import type { CardBodyProps } from '../OwnerFeed.type';
import { ActionBody } from './ActionBody';
import { AskBody } from './AskBody';
import { TestBody } from './TestBody';

const CardBody = (props: CardBodyProps) => {
  const { card, ctx, facts } = props;
  if (card.type === 'action') return <ActionBody action={card.data} actions={ctx.actions} />;
  if (card.type === 'test') return <TestBody facts={facts} build={card.data.build ?? null} handed={ctx.actions.handed} />;
  if (card.type === 'bundle') return <TestBody facts={facts} build={card.data.build} handed={ctx.actions.handed} />;
  return <AskBody entry={card.data} actions={ctx.actions} />;
};

export { CardBody };
