/* @layer renderer-app @kind component */
import { CopyButton } from '@drizztdourden08/tessera';
import { ChatButton, DrawerToolbar, useOpenThread } from '../../shared';
import { threadState } from '../behavior/thread-state';
import type { ThreadToolsProps } from '../ThreadDrawer.type';
import { ThreadLiveTools } from './ThreadLiveTools';
import { ThreadMenu } from './ThreadMenu';

const ThreadTools = (props: ThreadToolsProps) => {
  const { thread, paused, order, ops, wide, onWide, onPurge } = props;
  const state = threadState(thread);
  const open = useOpenThread();
  return (
    <DrawerToolbar id={thread.id} order={order} label="Thread tools" wide={wide} onWide={onWide} onOpen={open}>
      {state.live && <ThreadLiveTools thread={thread} paused={paused} ops={ops} />}
      {thread.link && <ChatButton link={thread.link} label={`Open the chat of ${thread.id}`} size="xs" />}
      {thread.link && <CopyButton text={thread.link} label="Copy the session link" variant="ghost" size="xs" />}
      <ThreadMenu thread={thread} paused={paused} ops={ops} onPurge={onPurge} />
    </DrawerToolbar>
  );
};

export { ThreadTools };
