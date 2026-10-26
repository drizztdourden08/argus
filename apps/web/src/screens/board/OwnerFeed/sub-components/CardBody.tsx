/* @layer renderer-app @kind component */
import type { FeedCardViewProps } from '../OwnerFeed.type';
import { ActionBody } from './ActionBody';
import { AskBody } from './AskBody';
import { TestBody } from './TestBody';

const CardBody = (props: FeedCardViewProps) => {
  const { card, actions } = props;
  if (card.type === 'action') return <ActionBody action={card.data} actions={actions} />;
  if (card.type === 'test') {
    return <TestBody build={card.data.build} title={card.data.title} summary={card.data.title} proofs={card.data.proofs} />;
  }
  if (card.type === 'bundle') {
    const { tests, build } = card.data;
    const count = `${tests.length} ${tests.length > 1 ? 'tests' : 'test'} to run`;
    const summary = `${count} · ${tests[0]?.title ?? ''}${tests.length > 1 ? ' ...' : ''}`;
    return <TestBody build={build} title={tests.map((test) => test.title).join('\n')} summary={summary} />;
  }
  return <AskBody entry={card.data} actions={actions} />;
};

export { CardBody };
