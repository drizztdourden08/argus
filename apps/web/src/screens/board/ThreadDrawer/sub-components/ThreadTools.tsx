/* @layer renderer-app @kind component */
import { Button, Flex } from '@drizztdourden08/tessera';
import { threadState } from '../behavior/thread-state';
import type { ThreadToolsProps } from '../ThreadDrawer.type';
import { ThreadChatTools } from './ThreadChatTools';
import { ThreadEndTools } from './ThreadEndTools';
import { ThreadLiveTools } from './ThreadLiveTools';

const ThreadTools = (props: ThreadToolsProps) => {
  const { thread, paused, ops, wide, onWide, onPurge } = props;
  const state = threadState(thread);
  return (
    <Flex gap="xs" align="center" wrap role="toolbar" aria-label="Thread tools">
      <Button size="xs" variant="ghost" aria-pressed={wide} title="Focus mode: the drawer takes the whole width" onClick={onWide}>
        {wide ? 'Collapse' : 'Expand'}
      </Button>
      {state.live && <ThreadLiveTools thread={thread} paused={paused} ops={ops} />}
      <ThreadChatTools thread={thread} />
      <ThreadEndTools thread={thread} state={state} ops={ops} onPurge={onPurge} />
    </Flex>
  );
};

export { ThreadTools };
