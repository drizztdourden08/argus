/* @layer renderer-app @kind component */
import { Button } from '@drizztdourden08/tessera';
import type { ThreadEndToolsProps } from '../ThreadDrawer.type';

const ThreadEndTools = (props: ThreadEndToolsProps) => {
  const { thread, state, ops, onPurge } = props;
  return (
    <>
      {(!state.coordinator || state.archived) && (
        <Button size="xs" variant="secondary" onClick={() => ops.soft(state.archived ? 'unarchive' : 'archive', thread)}>
          {state.archived ? 'Unarchive' : 'Archive'}
        </Button>
      )}
      {!state.coordinator && <Button size="xs" variant="danger" onClick={() => onPurge(thread.id)}>Purge...</Button>}
    </>
  );
};

export { ThreadEndTools };
