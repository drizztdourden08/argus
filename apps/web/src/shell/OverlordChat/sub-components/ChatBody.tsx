/* @layer renderer-app @kind component */
import { Flex } from '@drizztdourden08/tessera';
import { useChatFeed } from '../behavior/useChatFeed';
import { ChatBox } from './ChatBox';
import { TranscriptView } from './TranscriptView';

const ChatBody = () => {
  const feed = useChatFeed();
  return (
    <Flex direction="column" gap="lg">
      <TranscriptView rows={feed.rows} problem={feed.problem} />
      <ChatBox running={feed.running} action={feed.action} onSend={feed.onSend} />
    </Flex>
  );
};

export { ChatBody };
