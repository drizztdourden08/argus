/* @layer renderer-app @kind component */
import { Markdown, Text } from '@drizztdourden08/tessera';
import type { MessageBodyProps } from '../OverlordChat.type';

const MessageBody = (props: MessageBodyProps) => {
  const { role, content } = props;
  if (typeof content !== 'string') return <>{content}</>;
  if (role === 'system') return <Text variant="caption" tone="muted">{content}</Text>;
  return <Markdown size="sm" headingOffset={2} source={content} className="overlord-chat__text" />;
};

export { MessageBody };
