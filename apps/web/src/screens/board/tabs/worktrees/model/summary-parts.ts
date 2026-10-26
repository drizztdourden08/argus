/* @layer renderer-app @kind logic */
import type { WorktreesMeta } from '../../../../../data/worktrees';
import type { Counts, SummaryParts } from './model.type';
import { ruleProblems } from './rule-problems';

const plural = (count: number, one: string, many: string): string => `${count} ${count === 1 ? one : many}`;

const countsText = (counts: Counts): string => {
  if (counts.left === 0) return plural(counts.all, 'worktree', 'worktrees');
  return `${plural(counts.git, 'git worktree', 'git worktrees')}, ${plural(counts.left, 'leftover folder', 'leftover folders')}`;
};

const manyText = (meta: WorktreesMeta | null, counts: Counts): string | null => {
  if (!meta?.tooMany) return null;
  const tail = counts.left > 0 ? 'folders: that is a lot, delete the old leftovers' : 'worktrees: that is a lot, the thread should clean up';
  return `${meta.name ?? 'This project'} has ${counts.all} ${tail}`;
};

const goneText = (meta: WorktreesMeta | null): string | null => {
  if (!meta?.gone) return null;
  return `${meta.gone} gone (git still lists ${meta.gone === 1 ? 'it' : 'them'})`;
};

const summaryParts = (counts: Counts, meta: WorktreesMeta | null): SummaryParts => {
  const problems = ruleProblems(meta);
  return {
    counts: countsText(counts),
    need: `${counts.need} ${counts.need === 1 ? 'needs' : 'need'} a decision`,
    gone: goneText(meta),
    problems: problems > 0 ? plural(problems, 'rule problem', 'rule problems') : null,
    many: manyText(meta, counts),
  };
};

export { summaryParts };
