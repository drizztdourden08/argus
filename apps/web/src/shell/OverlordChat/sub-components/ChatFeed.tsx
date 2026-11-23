/* @layer renderer-app @kind component */
import { EmptyState, ScrollArea, Spinner, Stack, Text } from '@drizztdourden08/tessera';
import { EMPTY_TRANSCRIPT, WORKING_NOTE } from '../OverlordChat.constants';
import type { ChatFeedProps } from '../OverlordChat.type';
import { ChatMessage } from './ChatMessage';

const ChatFeed = (props: ChatFeedProps) => {
  const { messages, queued, working } = props;
  const all = [...messages, ...queued];
  if (all.length === 0 && !working) return <EmptyState message={EMPTY_TRANSCRIPT} className="overlord-chat__feed" />;
  return (
    <ScrollArea follow scrollbar="slim" className="overlord-chat__feed">
      <Stack gap="md" className="overlord-chat__messages">
        {all.map((message) => <ChatMessage key={message.id} message={message} />)}
        {working && (
          <Stack gap="xs" className="overlord-chat__working">
            <Spinner size="sm" label={WORKING_NOTE} />
            <Text variant="caption" tone="muted">{WORKING_NOTE}</Text>
          </Stack>
        )}
      </Stack>
    </ScrollArea>
  );
};

export { ChatFeed };
