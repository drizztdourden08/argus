/* @layer renderer-app @kind component */
import { Callout, EmptyState, Stack } from '@drizztdourden08/tessera';
import { Panel } from '../../Panel';
import { heldAt } from '../behavior/held-at';
import { heldText } from '../behavior/held-text';
import { queueTotal } from '../behavior/queue-total';
import { QUEUE_EMPTY, QUEUE_TITLES } from '../OverlordChat.constants';
import type { QueueFeedProps } from '../OverlordChat.type';
import { QueueLine } from './QueueLine';
import { QueueSection } from './QueueSection';

const QueueTab = (props: QueueFeedProps) => {
  const { queue, problem, action } = props.feed;
  const failure = problem ?? action.problem;
  if (!queue) return <Panel title="Queue">{failure && <Callout tone="danger">{failure}</Callout>}</Panel>;
  return (
    <Panel title="Queue">
      <Stack gap="md">
        {failure && <Callout tone="danger">{failure}</Callout>}
        {queueTotal(queue) === 0 && <EmptyState message={QUEUE_EMPTY} size="sm" />}
        {queueTotal(queue) > 0 && (
          <>
            <QueueSection title={QUEUE_TITLES.wakes} count={queue.wakes.length}>
              {queue.wakes.map((wake) => <QueueLine key={`${wake.at}${wake.reason}`} label={wake.reason} text={wake.text} at={wake.at} />)}
            </QueueSection>
            <QueueSection title={QUEUE_TITLES.held} count={queue.held.length}>
              {queue.held.map((held) => <QueueLine key={heldText(held)} text={heldText(held)} at={heldAt(held)} />)}
            </QueueSection>
            <QueueSection title={QUEUE_TITLES.alerts} count={queue.alerts.length}>
              {queue.alerts.map((alert) => <QueueLine key={`${alert.at}${alert.text}`} text={alert.text} at={alert.at} />)}
            </QueueSection>
          </>
        )}
      </Stack>
    </Panel>
  );
};

export { QueueTab };
