/* @layer renderer-app @kind component */
import { Button, CopyButton, Flex, Icon, IconButton } from '@drizztdourden08/tessera';
import { ChatButton, neighbours, useOpenThread } from '../../shared';
import { threadState } from '../behavior/thread-state';
import type { ThreadToolsProps } from '../ThreadDrawer.type';
import { ThreadLiveTools } from './ThreadLiveTools';
import { ThreadMenu } from './ThreadMenu';

const linkOf = (id: string): string => `${window.location.href.split('#')[0] ?? ''}#${id}`;

const ThreadTools = (props: ThreadToolsProps) => {
  const { thread, paused, order, ops, wide, onWide, onPurge } = props;
  const state = threadState(thread);
  const open = useOpenThread();
  const { previous, next } = neighbours(order, thread.id);
  return (
    <Flex gap="xs" align="center" wrap role="toolbar" aria-label="Thread tools">
      <CopyButton text={() => linkOf(thread.id)} label={`Copy a link to ${thread.id}`} variant="ghost" size="xs" />
      <IconButton label="Previous in the list ([)" size="xs" variant="ghost" disabled={previous === null} onClick={() => previous && open(previous)}>
        <Icon name="chevron-left" />
      </IconButton>
      <IconButton label="Next in the list (])" size="xs" variant="ghost" disabled={next === null} onClick={() => next && open(next)}>
        <Icon name="chevron-right" />
      </IconButton>
      <Button size="xs" variant="ghost" aria-pressed={wide} title="Focus mode: the drawer takes the whole width" onClick={onWide}>
        {wide ? 'Collapse' : 'Expand'}
      </Button>
      {state.live && <ThreadLiveTools thread={thread} paused={paused} ops={ops} />}
      {thread.link && <ChatButton link={thread.link} label={`Open the chat of ${thread.id}`} size="xs" />}
      {thread.link && <CopyButton text={thread.link} label="Copy the session link" variant="ghost" size="xs" />}
      <ThreadMenu thread={thread} paused={paused} ops={ops} onPurge={onPurge} />
    </Flex>
  );
};

export { ThreadTools };
