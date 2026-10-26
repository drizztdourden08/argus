/* @layer renderer-app @kind logic */
import type { Worktree } from '../../../../../data/worktrees';
import { SUBJECTS_SHOWN } from './model.constants';

const subjectsText = (subjects: string[], count: number | null): string => {
  if (subjects.length === 0) return '';
  const more = count !== null && count > subjects.length ? ` and ${count - subjects.length} more` : '';
  return `: ${subjects.map((subject) => `"${subject}"`).join(', ')}${more}`;
};

const commitsText = (row: Worktree): string => {
  if (row.merged || row.unmerged === 0) return '';
  const count = typeof row.unmerged === 'number' && Number.isInteger(row.unmerged) ? row.unmerged : null;
  const subjects = subjectsText((row.unmergedSubjects ?? []).slice(0, SUBJECTS_SHOWN), count);
  if (count === null) return `commits not known to be in the default branch${subjects}`;
  const into = row.mergedInto ?? 'the default branch';
  return `${count} ${count === 1 ? 'commit' : 'commits'} not merged into ${into}${subjects}`;
};

export { commitsText };
