/* @layer renderer-app @kind component */
import { EmptyState, ScrollArea, Stack } from '@drizztdourden08/tessera';
import { EMPTY_TRANSCRIPT } from '../OverlordChat.constants';
import type { ChatFeedProps } from '../OverlordChat.type';
import { ChatMessage } from './ChatMessage';
import { WorkingLine } from './WorkingLine';

const ChatFeed = (props: ChatFeedProps) => {
  const { messages, queued, working } = props;
  const all = [...messages, ...queued];
  if (all.length === 0 && working === null) return <EmptyState message={EMPTY_TRANSCRIPT} className="overlord-chat__feed" />;
  return (
    <ScrollArea follow scrollbar="slim" className="overlord-chat__feed">
      <Stack gap="md" className="overlord-chat__messages">
        {all.map((message) => <ChatMessage key={message.id} message={message} />)}
        {working && <WorkingLine activity={working} />}
      </Stack>
    </ScrollArea>
  );
};

export { ChatFeed };
