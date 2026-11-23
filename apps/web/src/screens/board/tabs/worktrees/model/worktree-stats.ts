/* @layer renderer-app @kind logic */
import type { WorktreesMeta } from '../../../../../data/worktrees';
import type { TabStat } from '../../../shared';
import type { Counts } from './model.type';
import { ruleProblemLines } from './rule-problem-lines';

const worktreeStats = (counts: Counts, meta: WorktreesMeta | null): TabStat[] => {
  const problems = ruleProblemLines(meta);
  const stats: TabStat[] = [
    { id: 'worktrees', label: 'worktrees', count: counts.git, tone: 'neutral' },
    { id: 'leftovers', label: 'leftovers', count: counts.left, tone: 'neutral', tip: 'Folders git no longer lists; only files are left.' },
    { id: 'need', label: 'need a decision', count: counts.need, tone: counts.need > 0 ? 'warning' : 'neutral' },
    { id: 'gone', label: 'gone', count: meta?.gone ?? 0, tone: meta?.gone ? 'warning' : 'neutral', tip: 'Git still lists them but their folders are gone.' },
  ];
  if (problems.length > 0) stats.push({ id: 'rules', label: 'rule problems', count: problems.length, tone: 'danger', tip: problems.join('\n') });
  if (meta?.tooMany) stats.push({ id: 'many', label: 'too many', count: counts.all, tone: 'warning', tip: 'That is a lot of folders: delete the old leftovers.' });
  return stats;
};

export { worktreeStats };
