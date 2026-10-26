/* @layer renderer-app @kind logic */
import type { BatchJob } from '../../../../../data/worktrees';
import { JOBS_SHOWN } from './model.constants';

const jobLine = (job: BatchJob): string => `${job.name ?? job.path} ${job.error ?? 'refused'}`;

const refusedText = (jobs: readonly BatchJob[]): string => {
  const listed = jobs.slice(0, JOBS_SHOWN).map(jobLine).join('; ');
  const more = jobs.length > JOBS_SHOWN ? ` and ${jobs.length - JOBS_SHOWN} more` : '';
  return `Not deleted: ${listed}${more}`;
};

export { refusedText };
