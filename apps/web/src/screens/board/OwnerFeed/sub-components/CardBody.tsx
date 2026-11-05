/* @layer renderer-app @kind component */
import type { FeedCardViewProps } from '../OwnerFeed.type';
import { ActionBody } from './ActionBody';
import { TestBody } from './TestBody';

const CardBody = (props: FeedCardViewProps) => {
  const { card, ctx } = props;
  if (card.type === 'action') return <ActionBody action={card.data} actions={ctx.actions} />;
  if (card.type === 'test') return <TestBody build={card.data.build ?? null} handed={ctx.actions.handed} />;
  if (card.type === 'bundle') return <TestBody build={card.data.build} handed={ctx.actions.handed} />;
  return null;
};

export { CardBody };
