/* @layer renderer-app @kind component */
import { Callout, ChatTranscript, Flex, Stack, Text } from '@drizztdourden08/tessera';
import { Panel } from '../../Panel';
import { heldText } from '../behavior/held-text';
import { useChatFeed } from '../behavior/useChatFeed';
import { EMPTY_TRANSCRIPT, WAITING_APPROVAL } from '../OverlordChat.constants';
import type { QueueFeedProps } from '../OverlordChat.type';
import { ApprovalLine } from './ApprovalLine';
import { ChatBox } from './ChatBox';

const ChatBody = (props: QueueFeedProps) => {
  const { feed: queueFeed } = props;
  const approvals = queueFeed.queue?.approvals ?? [];
  const held = (queueFeed.queue?.held ?? []).map(heldText);
  const feed = useChatFeed(held, queueFeed.reload);
  return (
    <Flex direction="column" gap="lg">
      <Panel title="Transcript">
        {feed.problem && <Callout tone="danger">{feed.problem}</Callout>}
        <ChatTranscript messages={feed.messages} empty={feed.problem ? undefined : EMPTY_TRANSCRIPT} className="overlord-transcript" />
      </Panel>
      {approvals.length > 0 && (
        <Stack gap="sm">
          <Text variant="label">{WAITING_APPROVAL}</Text>
          {approvals.map((approval) => (
            <ApprovalLine key={approval.id} approval={approval} busy={queueFeed.action.busy} onDecide={(id, decision) => void queueFeed.decide(id, decision)} />
          ))}
          {queueFeed.action.problem && <Callout tone="danger">{queueFeed.action.problem}</Callout>}
        </Stack>
      )}
      {feed.queued.length > 0 && <ChatTranscript messages={feed.queued} />}
      <ChatBox running={feed.running} note={feed.note} action={feed.action} onSend={feed.onSend} />
    </Flex>
  );
};

export { ChatBody };
