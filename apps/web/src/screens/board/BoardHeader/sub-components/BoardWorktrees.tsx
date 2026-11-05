/* @layer renderer-app @kind component */
import { Icon, Link, Text } from '@drizztdourden08/tessera';
import { SLACK } from '../../../../shell/WorktreeCounter/WorktreeCounter.constants';
import { WORKTREES_HINT, WORKTREES_OVER_HINT } from '../BoardHeader.constants';
import type { BoardWorktreesProps } from '../BoardHeader.type';

const BoardWorktrees = (props: BoardWorktreesProps) => {
  const { listed, runningAgents } = props;
  const over = listed > runningAgents + SLACK;
  return (
    <Text variant="caption" tone={over ? 'warning' : 'muted'}>
      {over ? <Icon name="triangle-alert" size={12} aria-hidden /> : null}
      <Link href="#worktrees" variant="subtle" title={over ? WORKTREES_OVER_HINT : WORKTREES_HINT}>{`${listed} worktrees`}</Link>
    </Text>
  );
};

export { BoardWorktrees };
