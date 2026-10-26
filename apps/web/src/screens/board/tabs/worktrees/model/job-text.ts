/* @layer renderer-app @kind logic */
import type { WorktreeJob } from '../../../../../data/worktrees';

const runningText = (job: WorktreeJob): string => `deleting, ${job.removed ?? 0}${job.total ? ` of ${job.total}` : ''} entries`;

const jobText = (job: WorktreeJob): string => {
  if (job.state === 'failed') return `failed: ${job.error ?? 'error'} (finished at the next start)`;
  if (job.state === 'running') return runningText(job);
  if (job.state === 'checking') return 'checking it again';
  if (job.state === 'queued') return 'waiting for its turn';
  return job.state;
};

export { jobText };
