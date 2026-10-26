/* @layer renderer-app @kind component */
import { DateText, Link, ListItemList, ListItemRow, Status, Tag, Text } from '@drizztdourden08/tessera';
import { CHAT_SHAPE } from '../AgentsTab.constants';
import type { ChatsListProps } from '../AgentsTab.type';
import { activityTone, orText } from '../model';

const ChatsList = (props: ChatsListProps) => {
  const { chats } = props;
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
          action={chat.link ? <Link href={chat.link} variant="subtle" external>Open chat</Link> : <Text tone="muted">-</Text>}
          data={{ 'data-chat-id': chat.id }}
        />
      ))}
    </ListItemList>
  );
};

export { ChatsList };
