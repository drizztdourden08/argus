/* @layer renderer-app @kind logic */
import type { WorktreeRule, WorktreesMeta } from '../../../../../data/worktrees';
import { DEFAULT_LIMIT_DAYS } from './model.constants';

const foldersText = (candidates: number | null | undefined): string => {
  if (typeof candidates !== 'number') return '';
  return `, ${candidates} ${candidates === 1 ? 'folder' : 'folders'}`;
};

const ruleDetails = (rule: WorktreeRule, meta: WorktreesMeta | null): string => {
  const stale = rule.staleDays ?? meta?.staleDays ?? DEFAULT_LIMIT_DAYS;
  return `tells ${rule.thread ?? '?'}, stale after ${stale} days${foldersText(rule.candidates)}`;
};

export { ruleDetails };
