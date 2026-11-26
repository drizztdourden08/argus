/* @layer renderer-app @kind component */
import { Callout, Composer, Flex, Stack, Text } from '@drizztdourden08/tessera';
import { useState } from 'react';
import { CHAT_LABEL, CHAT_PLACEHOLDER } from '../OverlordChat.constants';
import type { ChatBoxProps } from '../OverlordChat.type';

const ChatBox = (props: ChatBoxProps) => {
  const { running, note, action, bar, onSend } = props;
  const [text, setText] = useState('');
  const send = (value: string) => {
    void onSend(value).then((sent) => sent && setText(''));
  };
  return (
    <Stack gap="sm">
      {!running && <Text variant="caption" tone="muted">{note}</Text>}
      <Stack gap="xs">
        <Composer
          aria-label={CHAT_LABEL}
          value={text}
          onChange={setText}
          onSubmit={send}
          disabled={!running}
          busy={action.busy}
          placeholder={CHAT_PLACEHOLDER}
        />
        <Flex gap="xs" align="center" className="overlord-chat__bar">{bar}</Flex>
      </Stack>
      {action.problem && <Callout tone="danger">{action.problem}</Callout>}
    </Stack>
  );
};

export { ChatBox };
