/* @layer renderer-app @kind component */
import { CopyButton, Divider, Flex, Icon, IconButton, Inline, Spacer, Text } from '@drizztdourden08/tessera';
import { useNavigate } from 'react-router';
import type { CardFooterProps } from '../dashboards.type';

const CardFooter = (props: CardFooterProps) => {
  const { card, started, onSend } = props;
  const navigate = useNavigate();
  return (
    <Flex direction="column" gap="xs">
      <Divider />
      <Inline gap="2xs" align="center">
        <IconButton size="sm" variant="ghost" label={`Open the ${card.name} board`} disabled={!started} title={started ? `Open the ${card.name} board` : 'No board yet'} onClick={() => void navigate(card.url)}>
          <Icon name="external-link" />
        </IconButton>
        <IconButton size="sm" variant="ghost" label="Send down" disabled={!started} title={started ? `Send a decision or a note to ${card.name}` : 'No board yet: nobody to send to'} onClick={() => onSend(card.key)}>
          <Icon name="send" />
        </IconButton>
        <CopyButton text={card.key} label="Copy the project key" size="sm" variant="ghost" />
        <Spacer />
        <Text variant="caption" mono title={card.key}>{card.key}</Text>
      </Inline>
    </Flex>
  );
};

export { CardFooter };
