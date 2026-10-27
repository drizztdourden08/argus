/* @layer renderer-app @kind component */
import { Box, Text } from '@drizztdourden08/tessera';
import type { ChatsListProps } from '../AgentsTab.type';
import { ChatRow } from './ChatRow';

const ChatsList = (props: ChatsListProps) => {
  const { boardKey, chats, now, onPurge } = props;
  if (chats.length === 0) return null;
  return (
    <Box as="section" aria-label="Chats" className="agents-chats">
      <Text as="h3" variant="overline" className="agents-chats__title">{`Chats · ${chats.length}`}</Text>
      {chats.map((chat) => <ChatRow key={chat.id} boardKey={boardKey} chat={chat} now={now} onPurge={onPurge} />)}
    </Box>
  );
};

export { ChatsList };
