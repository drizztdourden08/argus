/* @layer renderer-app @kind logic */
import type { TabStat } from '../../../shared';
import { FAILING_TIP, NOT_ON_GITHUB_TIP } from '../changes.constants';
import type { PullRequestDoc } from '../changes.type';

const countWhere = (prs: readonly PullRequestDoc[], status: string): number => prs.filter((pr) => pr.status === status).length;

const changesStats = (prs: readonly PullRequestDoc[], failing: ReadonlySet<string>): TabStat[] => [
  { id: 'open', label: 'Open', count: countWhere(prs, 'open'), tone: 'info' },
  { id: 'failing', label: 'Failing', count: failing.size, tone: 'danger', tip: FAILING_TIP },
  { id: 'merged', label: 'Merged', count: countWhere(prs, 'merged'), tone: 'success' },
  { id: 'closed', label: 'Closed', count: countWhere(prs, 'closed'), tone: 'neutral' },
  { id: 'not-on-github', label: 'Not on GitHub', count: prs.filter((pr) => pr.replay?.state !== 'done').length, tone: 'warning', tip: NOT_ON_GITHUB_TIP },
];

export { changesStats };
