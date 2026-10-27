/* @layer renderer-app @kind component */
import { Box, Pressable, Status, Tag } from '@drizztdourden08/tessera';
import { formatAgo } from '../../../../../format';
import { ThreadMenu, useThreadOps, useThreadView } from '../../../ThreadDrawer';
import { ChatButton, activityTone, useOpenThread } from '../../../shared';
import type { ChatRowProps } from '../AgentsTab.type';
import { orText } from '../model';

const ChatRow = (props: ChatRowProps) => {
  const { boardKey, chat, now, onPurge } = props;
  const open = useOpenThread();
  const ops = useThreadOps(boardKey);
  const view = useThreadView(boardKey, chat.id);
  const seen = chat.lastSeen ? `${formatAgo(chat.lastSeen, now)} ago` : 'never';
  return (
    <Box className={`agents-chat${chat.status === 'paused' ? ' agents-chat--paused' : ''}`} data-chat-id={chat.id}>
      <Box as="span" className="agents-chat__name">
        <Status tone={activityTone(chat.activity)} dot title={orText(chat.activity, 'unknown')} aria-label={orText(chat.activity, 'unknown')}>{null}</Status>
        <Pressable className="agents-name__link" title={`${orText(chat.session, 'no session title')}\nSubject: ${orText(chat.subject, 'none')}\nSeen: ${seen}`} onClick={() => open(chat.id)}>
          {chat.id}
        </Pressable>
      </Box>
      <Tag variant="normal" color="neutral">{chat.status}</Tag>
      <Pressable className="agents-chat__note" title={orText(chat.note, 'no note')} onClick={() => open(chat.id)}>
        {orText(chat.note, '-')}
      </Pressable>
      {chat.link && <ChatButton link={chat.link} label={`Open the chat of ${chat.id}`} />}
      <ThreadMenu thread={chat} paused={view.paused} ops={ops} onPurge={onPurge} onEditSubject={() => open(chat.id)} />
    </Box>
  );
};

export { ChatRow };
