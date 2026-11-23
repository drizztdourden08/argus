/* @layer renderer-app @kind component */
import { Button, Inline } from '@drizztdourden08/tessera';
import { REVIEW_LABEL, UNDO_LABEL } from '../compare.constants';
import type { SyncActionsProps } from '../cards.type';

const SyncActions = (props: SyncActionsProps) => {
  const { run, actions, onReview } = props;
  const finished = run?.phase === 'done' || run?.phase === 'failed';
  return (
    <Inline gap="xs" wrap>
      <Button size="xs" variant="ghost" onClick={onReview}>{REVIEW_LABEL}</Button>
      {finished && <Button size="xs" variant="ghost" disabled={actions.busy} onClick={actions.undo}>{UNDO_LABEL}</Button>}
    </Inline>
  );
};

export { SyncActions };
