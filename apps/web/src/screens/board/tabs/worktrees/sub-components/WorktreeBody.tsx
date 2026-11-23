/* @layer renderer-app @kind component */
import { Button, EmptyState } from '@drizztdourden08/tessera';
import type { WorktreeBodyProps } from '../worktrees-props.type';
import { WorktreeGroups } from './WorktreeGroups';

const WorktreeBody = (props: WorktreeBodyProps) => {
  const { total, needOnly, hasRules, onShowAll, ...table } = props;
  if (!hasRules) return <EmptyState message="The thread has not told Argus where its worktrees live yet." />;
  if (total === 0) {
    const hint = table.meta?.scannedAt ? undefined : 'The first scan has not finished yet.';
    return <EmptyState message="No worktrees found under the rules." hint={hint} />;
  }
  if (table.rows.length === 0) {
    if (!needOnly) return <EmptyState size="sm" message="No worktree matches the filters." />;
    return <EmptyState size="sm" message="Nothing needs a decision." action={<Button size="sm" variant="ghost" onClick={onShowAll}>{`Show all ${total}`}</Button>} />;
  }
  return <WorktreeGroups {...table} />;
};

export { WorktreeBody };
