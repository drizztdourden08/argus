/* @layer renderer-app @kind component */
import { Button } from '@drizztdourden08/tessera';
import { useSwitchActions } from '../behavior/useSwitchActions';
import { isThreadPending } from '../model';
import { SKIP_THREAD_LABEL } from '../switch.constants';
import type { ThreadRowProps } from '../switch.type';

const ThreadSkip = (props: ThreadRowProps) => {
  const { thread, skippable } = props;
  const actions = useSwitchActions();
  const { board, thread: id } = thread;
  if (!skippable || !isThreadPending(thread) || !board || !id) return null;
  return (
    <Button size="xs" variant="ghost" disabled={actions.busy} onClick={() => actions.skipThread(board, id)}>{SKIP_THREAD_LABEL}</Button>
  );
};

export { ThreadSkip };
