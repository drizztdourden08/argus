/* @layer renderer-app @kind logic */
import type { BatchJob, WorktreesMeta } from '../../../../../data/worktrees';
import { JOB_WINDOW_MS } from './model.constants';

const isRecent = (job: BatchJob, now: number): boolean => now - Date.parse(job.finishedAt ?? '') < JOB_WINDOW_MS;

const refusedJobs = (meta: WorktreesMeta | null, dismissed: ReadonlySet<string>, now: number): BatchJob[] =>
  (meta?.jobs ?? []).filter((job) => job.state === 'refused' && !dismissed.has(job.id) && isRecent(job, now));

export { refusedJobs };
