/* @layer renderer-app @kind component */
import { Button, Callout, Flex, Stack, Text, Textarea } from '@drizztdourden08/tessera';
import { useState } from 'react';
import { CHAT_PLACEHOLDER, STOPPED_CHAT } from '../OverlordChat.constants';
import type { ChatBoxProps } from '../OverlordChat.type';

const ChatBox = (props: ChatBoxProps) => {
  const { running, action, onSend } = props;
  const [text, setText] = useState('');
  const blocked = !running || action.busy || text.trim() === '';
  const send = () => {
    if (!blocked) void onSend(text).then((sent) => sent && setText(''));
  };
  return (
    <Stack gap="sm">
      {!running && <Text variant="caption" tone="muted">{STOPPED_CHAT}</Text>}
      <Flex gap="sm" align="end">
        <Textarea
          aria-label="Message to the Overlord"
          rows={2}
          value={text}
          disabled={!running}
          placeholder={CHAT_PLACEHOLDER}
          onChange={(event) => setText(event.target.value)}
        />
        <Button variant="primary" disabled={blocked} onClick={send}>Send</Button>
      </Flex>
      {action.problem && <Callout tone="danger">{action.problem}</Callout>}
    </Stack>
  );
};

export { ChatBox };
