/* @layer renderer-app @kind component */
import { CopyButton, Divider, Flex, Icon, IconButton, Inline, Spacer, Text } from '@drizztdourden08/tessera';
import { useRef } from 'react';
import { useNavigate } from 'react-router';
import type { CardFooterProps } from '../dashboards.type';
import { SendDownPopover } from './SendDownPopover';

const CardFooter = (props: CardFooterProps) => {
  const { card, started, send } = props;
  const navigate = useNavigate();
  const anchor = useRef<HTMLButtonElement>(null);
  return (
    <Flex direction="column" gap="xs">
      <Divider />
      <Inline gap="2xs" align="center">
        <IconButton size="sm" variant="ghost" label={`Open the ${card.name} board`} disabled={!started} title={started ? `Open the ${card.name} board` : 'No board yet'} onClick={() => void navigate(card.url)}>
          <Icon name="external-link" />
        </IconButton>
        <IconButton ref={anchor} size="sm" variant="ghost" label="Send down" disabled={!started} title={started ? `Send a decision or a note to ${card.name}` : 'No board yet: nobody to send to'} onClick={() => send.open(card.key)}>
          <Icon name="send" />
        </IconButton>
        <SendDownPopover send={send} owner={card.key} anchorRef={anchor} />
        <CopyButton text={card.key} label="Copy the project key" size="sm" variant="ghost" />
        <Spacer />
        <Text variant="caption" mono title={card.key}>{card.key}</Text>
      </Inline>
    </Flex>
  );
};

export { CardFooter };
