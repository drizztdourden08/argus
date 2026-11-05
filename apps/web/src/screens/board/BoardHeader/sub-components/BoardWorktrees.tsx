/* @layer renderer-app @kind component */
import { Icon, Link, Text } from '@drizztdourden08/tessera';
import { useDoc } from '../../../../data/react';
import type { WorktreesMeta } from '../../../../data/worktrees';
import { SLACK } from '../../../../shell/WorktreeCounter/WorktreeCounter.constants';
import { RUNNING_HINT, WORKTREES_HINT, WORKTREES_OVER_HINT } from '../BoardHeader.constants';
import type { BoardWorktreesProps } from '../BoardHeader.type';

const BoardWorktrees = (props: BoardWorktreesProps) => {
  const { doc } = useDoc<WorktreesMeta>(props.boardKey, 'meta', 'worktrees');
  if (typeof doc?.listed !== 'number' || typeof doc.runningAgents !== 'number') return null;
  const over = doc.listed > doc.runningAgents + SLACK;
  return (
    <>
      <Text variant="caption" tone={over ? 'warning' : 'muted'}>
        {over ? <Icon name="triangle-alert" size={12} aria-hidden /> : null}
        <Link href="#worktrees" variant="subtle" title={over ? WORKTREES_OVER_HINT : WORKTREES_HINT}>{`${doc.listed} worktrees`}</Link>
      </Text>
      <Text variant="caption" tone="muted">
        <Link href="#agents" variant="subtle" title={RUNNING_HINT}>{`${doc.runningAgents} running`}</Link>
      </Text>
    </>
  );
};

export { BoardWorktrees };
