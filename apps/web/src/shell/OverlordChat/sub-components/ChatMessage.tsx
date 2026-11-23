/* @layer renderer-app @kind component */
import { Stack, Text } from '@drizztdourden08/tessera';
import type { ChatMessageProps } from '../OverlordChat.type';
import { MessageBody } from './MessageBody';

const ChatMessage = (props: ChatMessageProps) => {
  const { message } = props;
  const role = message.role ?? 'assistant';
  return (
    <Stack gap="2xs" className={`overlord-chat__message is-${role}`}>
      {message.content != null && <MessageBody role={role} content={message.content} />}
      {message.time != null && <Text variant="caption" tone="faint">{message.time}</Text>}
    </Stack>
  );
};

export { ChatMessage };
