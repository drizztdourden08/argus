/* @layer renderer-app @kind component */
import { Button } from '@drizztdourden08/tessera';
import type { EntryBodyProps } from '../OwnerFeed.type';

const DismissButton = (props: EntryBodyProps) => {
  const { entry, actions } = props;
  return (
    <Button size="sm" variant="ghost" disabled={actions.isBusy(`post:ack:${entry.id}`)} onClick={() => void actions.dismiss(entry)}>
      Dismiss
    </Button>
  );
};

export { DismissButton };
