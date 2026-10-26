/* @layer renderer-app @kind component */
import { Button } from '@drizztdourden08/tessera';
import { FOCUS_KEY, useStored } from '../../shared';
import type { ThreadLiveToolsProps } from '../ThreadDrawer.type';

const ThreadLiveTools = (props: ThreadLiveToolsProps) => {
  const { thread, paused, ops } = props;
  const [focus, setFocus] = useStored(FOCUS_KEY);
  const focused = focus === thread.id;
  return (
    <>
      <Button
        size="xs"
        variant="ghost"
        aria-pressed={focused}
        title={focused ? 'Showing only this thread, click to show all' : 'Focus on this thread'}
        onClick={() => setFocus(focused ? '' : thread.id)}
      >
        Focus
      </Button>
      <Button size="xs" variant="secondary" disabled={ops.isBusy(`post:${paused ? 'resume' : 'pause'}:${thread.id}`)} onClick={() => ops.setPaused(thread, !paused)}>
        {paused ? 'Resume' : 'Pause'}
      </Button>
    </>
  );
};

export { ThreadLiveTools };
