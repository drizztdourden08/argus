/* @layer renderer-app @kind component */
import { DateText, Flex, Link, ListItemList, ListItemRow, Status, Tag, Text } from '@drizztdourden08/tessera';
import { CHAT_SHAPE } from '../AgentsTab.constants';
import type { ChatsListProps } from '../AgentsTab.type';
import { activityTone, orText } from '../model';
import { ChatMenu } from './ChatMenu';

const ChatsList = (props: ChatsListProps) => {
  const { chats, onPurge } = props;
  if (chats.length === 0) return null;
  return (
    <ListItemList label="Chats" heading="Chats" count={chats.length} shape={CHAT_SHAPE}>
      {chats.map((chat) => (
        <ListItemRow
          key={chat.id}
          name={chat.id}
          meta={orText(chat.note, 'no note')}
          columns={[
            {
              primary: <Status tone={activityTone(chat.activity)} dot>{orText(chat.activity, 'unknown')}</Status>,
              secondary: chat.lastSeen ? <DateText value={chat.lastSeen} format="relative" /> : undefined,
            },
            { primary: <Tag variant="normal" color="neutral">{chat.status}</Tag> },
          ]}
          action={
            <Flex gap="xs" align="center">
              {chat.link ? <Link href={chat.link} variant="subtle" external>Open chat</Link> : <Text tone="muted">-</Text>}
              <ChatMenu id={chat.id} onPurge={onPurge} />
            </Flex>
          }
          data={{ 'data-chat-id': chat.id }}
        />
      ))}
    </ListItemList>
  );
};

export { ChatsList };
