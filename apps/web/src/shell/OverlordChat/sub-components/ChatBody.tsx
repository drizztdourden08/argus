/* @layer renderer-app @kind component */
import { Callout, Flex } from '@drizztdourden08/tessera';
import { heldText } from '../behavior/held-text';
import { useChatSettings } from '../behavior/useChatSettings';
import { useChatUsage } from '../behavior/useChatUsage';
import { useChatFeed } from '../behavior/useChatFeed';
import type { QueueFeedProps } from '../behavior/chat-panels.type';
import { ChatBar } from './ChatBar';
import { ChatBox } from './ChatBox';
import { ChatFeed } from './ChatFeed';

const ChatBody = (props: QueueFeedProps) => {
  const { feed: queueFeed } = props;
  const held = (queueFeed.queue?.held ?? []).map(heldText);
  const feed = useChatFeed(held, queueFeed.reload);
  const chat = useChatSettings();
  const usage = useChatUsage(feed.working !== null);
  return (
    <Flex direction="column" gap="md" className="overlord-chat__pane">
      {feed.problem && <Callout tone="danger">{feed.problem}</Callout>}
      <ChatFeed messages={feed.messages} queued={feed.queued} working={feed.working} />
      <ChatBox running={feed.running} note={feed.note} action={feed.action} bar={<ChatBar chat={chat} usage={usage} />} onSend={feed.onSend} />
      {chat.problem && <Callout tone="danger">{chat.problem}</Callout>}
    </Flex>
  );
};

export { ChatBody };
