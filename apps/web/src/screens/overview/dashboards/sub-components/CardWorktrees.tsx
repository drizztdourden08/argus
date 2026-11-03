/* @layer renderer-app @kind component */
import { PORTFOLIO_SCOPE, useDoc } from '../../../../data/react';
import type { PortfolioWorktrees } from '../../../../data/worktrees';
import { WorktreeCounter } from '../../../../shell';
import type { CardCountersProps } from '../dashboards.type';

const CardWorktrees = (props: CardCountersProps) => {
  const { doc } = useDoc<PortfolioWorktrees>(PORTFOLIO_SCOPE, 'meta', 'worktrees');
  const counter = doc?.counts?.[props.card.key];
  if (!counter) return null;
  return <WorktreeCounter listed={counter.listed} running={counter.running} />;
};

export { CardWorktrees };
