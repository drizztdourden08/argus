/* @layer renderer-app @kind component */
import { Button, Inline } from '@drizztdourden08/tessera';
import { REVIEW_LABEL, SYNC_LABEL, UNDO_LABEL } from '../compare.constants';
import type { SyncActionsProps } from '../cards.type';

const SyncActions = (props: SyncActionsProps) => {
  const { run, active, actions, onReview } = props;
  const finished = run?.phase === 'done' || run?.phase === 'failed';
  return (
    <Inline gap="xs" wrap>
      <Button size="xs" variant="ghost" onClick={onReview}>{REVIEW_LABEL}</Button>
      {finished && <Button size="xs" variant="ghost" disabled={actions.busy} onClick={actions.undo}>{UNDO_LABEL}</Button>}
      <Button size="xs" variant="primary" disabled={actions.busy || active} onClick={actions.sync}>{SYNC_LABEL}</Button>
    </Inline>
  );
};

export { SyncActions };
