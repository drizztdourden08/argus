/* @layer renderer-app @kind component */
import { Stack, Text } from '@drizztdourden08/tessera';
import type { ChatMessageProps } from '../OverlordChat.type';

const ChatMessage = (props: ChatMessageProps) => {
  const { message } = props;
  const role = message.role ?? 'assistant';
  return (
    <Stack gap="2xs" className={`overlord-chat__message is-${role}`}>
      {message.content != null && (role === 'system' ? <Text variant="caption" tone="muted">{message.content}</Text> : <div className="overlord-chat__text">{message.content}</div>)}
      {message.tools?.map((tool) => (
        <Text key={tool.id} variant="caption" tone="muted" className="overlord-chat__tool">
          {tool.name} {tool.summary}{tool.status === 'failed' ? ' (failed)' : ''}
        </Text>
      ))}
      {message.time != null && <Text variant="caption" tone="faint">{message.time}</Text>}
    </Stack>
  );
};

export { ChatMessage };
