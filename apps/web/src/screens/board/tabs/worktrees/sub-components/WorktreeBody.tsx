/* @layer renderer-app @kind component */
import { Button, EmptyState } from '@drizztdourden08/tessera';
import { NO_RULES_HINT } from '../worktrees.constants';
import type { WorktreeBodyProps } from '../worktrees-props.type';
import { WorktreeGroups } from './WorktreeGroups';

const WorktreeBody = (props: WorktreeBodyProps) => {
  const { total, filter, hasRules, onShowAll, ...table } = props;
  if (!hasRules) return <EmptyState message="The thread has not told Argus where its worktrees live yet." hint={NO_RULES_HINT} />;
  if (total === 0) {
    const hint = table.meta?.scannedAt ? undefined : 'The first scan has not finished yet.';
    return <EmptyState message="No worktrees found under the rules." hint={hint} />;
  }
  if (table.rows.length === 0) {
    if (filter === 'all') return <EmptyState size="sm" message="No worktree matches the filters." />;
    const message = filter === 'left' ? 'No leftover folders.' : 'Nothing needs a decision.';
    return <EmptyState size="sm" message={message} action={<Button size="sm" variant="ghost" onClick={onShowAll}>{`Show all ${total}`}</Button>} />;
  }
  return <WorktreeGroups {...table} />;
};

export { WorktreeBody };
