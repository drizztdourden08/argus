/* @layer renderer-app @kind component */
import { Button, Icon, IconButton } from '@drizztdourden08/tessera';
import { focusKey, useStored } from '../../shared';
import type { ThreadLiveToolsProps } from '../ThreadDrawer.type';

const ThreadLiveTools = (props: ThreadLiveToolsProps) => {
  const { boardKey, thread, paused, ops } = props;
  const [focus, setFocus] = useStored(focusKey(boardKey));
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
      <IconButton
        label={paused ? 'Resume this thread' : 'Pause this thread'}
        size="xs"
        variant="ghost"
        disabled={ops.isBusy(`post:${paused ? 'resume' : 'pause'}:${thread.id}`)}
        onClick={() => ops.setPaused(thread, !paused)}
      >
        <Icon name={paused ? 'play' : 'pause'} />
      </IconButton>
    </>
  );
};

export { ThreadLiveTools };
