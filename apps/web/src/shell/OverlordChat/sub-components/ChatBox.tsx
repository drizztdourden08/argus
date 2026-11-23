/* @layer renderer-app @kind component */
import { Callout, Flex, Icon, IconButton, Stack, Text, Textarea } from '@drizztdourden08/tessera';
import { useState } from 'react';
import { CHAT_PLACEHOLDER, SEND_LABEL } from '../OverlordChat.constants';
import type { ChatBoxProps } from '../OverlordChat.type';

const ChatBox = (props: ChatBoxProps) => {
  const { running, note, action, onSend } = props;
  const [text, setText] = useState('');
  const blocked = !running || action.busy || text.trim() === '';
  const send = () => {
    if (!blocked) void onSend(text).then((sent) => sent && setText(''));
  };
  return (
    <Stack gap="sm">
      {!running && <Text variant="caption" tone="muted">{note}</Text>}
      <Flex gap="sm" align="end" className="overlord-chat__box">
        <Textarea
          aria-label="Message to the Overlord"
          className="overlord-chat__input"
          rows={1}
          resize="none"
          value={text}
          disabled={!running}
          placeholder={CHAT_PLACEHOLDER}
          onChange={(event) => setText(event.target.value)}
          onKeyDown={(event) => {
            if (event.key !== 'Enter' || event.shiftKey || event.nativeEvent.isComposing) return;
            event.preventDefault();
            send();
          }}
        />
        <IconButton label={SEND_LABEL} variant="primary" size="sm" disabled={blocked} onClick={send}>
          <Icon name="arrow-up" />
        </IconButton>
      </Flex>
      {action.problem && <Callout tone="danger">{action.problem}</Callout>}
    </Stack>
  );
};

export { ChatBox };
