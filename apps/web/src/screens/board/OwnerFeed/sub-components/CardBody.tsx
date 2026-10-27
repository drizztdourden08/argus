/* @layer renderer-app @kind component */
import type { FeedCardViewProps } from '../OwnerFeed.type';
import { ActionBody } from './ActionBody';
import { AskBody } from './AskBody';

const CardBody = (props: FeedCardViewProps) => {
  const { card, actions } = props;
  if (card.type === 'action') return <ActionBody action={card.data} actions={actions} />;
  if (card.type === 'test' || card.type === 'bundle') return null;
  return <AskBody entry={card.data} actions={actions} />;
};

export { CardBody };
