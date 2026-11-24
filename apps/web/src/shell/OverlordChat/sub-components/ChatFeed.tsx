/* @layer renderer-app @kind component */
import { ChatTranscript, EmptyState, Flex } from '@drizztdourden08/tessera';
import type { ChatTranscriptMessage } from '@drizztdourden08/tessera';
import { EMPTY_TRANSCRIPT } from '../OverlordChat.constants';
import type { ChatFeedProps } from '../OverlordChat.type';
import { MessageBody } from './MessageBody';
import { WorkingLine } from './WorkingLine';

const withBody = (message: ChatTranscriptMessage): ChatTranscriptMessage =>
  message.content == null ? message : { ...message, content: <MessageBody role={message.role ?? 'assistant'} content={message.content} /> };

const ChatFeed = (props: ChatFeedProps) => {
  const { messages, queued, working } = props;
  const all = [...messages, ...queued].map(withBody);
  if (all.length === 0 && working === null) return <EmptyState message={EMPTY_TRANSCRIPT} className="overlord-chat__feed" />;
  return (
    <Flex direction="column" gap="sm" className="overlord-chat__feed">
      <ChatTranscript messages={all} className="overlord-chat__transcript" />
      {working && <WorkingLine activity={working} />}
    </Flex>
  );
};

export { ChatFeed };
