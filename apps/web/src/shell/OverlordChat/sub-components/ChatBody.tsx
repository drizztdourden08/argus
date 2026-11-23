/* @layer renderer-app @kind component */
import { Callout, Flex } from '@drizztdourden08/tessera';
import { heldText } from '../behavior/held-text';
import { useChatFeed } from '../behavior/useChatFeed';
import type { QueueFeedProps } from '../OverlordChat.type';
import { ChatBar } from './ChatBar';
import { ChatBox } from './ChatBox';
import { ChatFeed } from './ChatFeed';

const ChatBody = (props: QueueFeedProps) => {
  const { feed: queueFeed } = props;
  const held = (queueFeed.queue?.held ?? []).map(heldText);
  const feed = useChatFeed(held, queueFeed.reload);
  return (
    <Flex direction="column" gap="md" className="overlord-chat__pane">
      {feed.problem && <Callout tone="danger">{feed.problem}</Callout>}
      <ChatFeed messages={feed.messages} queued={feed.queued} working={feed.working} />
      <ChatBox running={feed.running} note={feed.note} action={feed.action} onSend={feed.onSend} />
      <ChatBar working={feed.working} />
    </Flex>
  );
};

export { ChatBody };
