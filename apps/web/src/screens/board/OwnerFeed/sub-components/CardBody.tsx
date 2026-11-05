/* @layer renderer-app @kind component */
import type { FeedCardViewProps } from '../OwnerFeed.type';
import { ActionBody } from './ActionBody';
import { BuildTag } from './BuildTag';
import { RunBody } from './RunBody';

const CardBody = (props: FeedCardViewProps) => {
  const { card, ctx } = props;
  if (card.type === 'action') return <ActionBody action={card.data} actions={ctx.actions} />;
  if (card.type === 'test') return <BuildTag build={card.data.build ?? null} />;
  if (card.type === 'run') return <RunBody run={card.data} />;
  return null;
};

export { CardBody };
