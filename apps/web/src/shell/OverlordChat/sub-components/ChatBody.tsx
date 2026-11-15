/* @layer renderer-app @kind component */
import { Callout, ChatTranscript, Flex } from '@drizztdourden08/tessera';
import { Panel } from '../../Panel';
import { useChatFeed } from '../behavior/useChatFeed';
import { EMPTY_TRANSCRIPT } from '../OverlordChat.constants';
import { ChatBox } from './ChatBox';

const ChatBody = () => {
  const feed = useChatFeed();
  return (
    <Flex direction="column" gap="lg">
      <Panel title="Transcript">
        {feed.problem && <Callout tone="danger">{feed.problem}</Callout>}
        <ChatTranscript messages={feed.messages} empty={feed.problem ? undefined : EMPTY_TRANSCRIPT} className="overlord-transcript" />
      </Panel>
      <ChatBox running={feed.running} action={feed.action} onSend={feed.onSend} />
    </Flex>
  );
};

export { ChatBody };
