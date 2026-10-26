/* @layer renderer-app @kind component */
import { Button, Callout } from '@drizztdourden08/tessera';
import { refusedText } from '../model';
import type { RefusedJobsProps } from '../worktrees-props.type';

const RefusedJobs = (props: RefusedJobsProps) => {
  const { jobs, onDismiss } = props;
  if (jobs.length === 0) return null;
  return (
    <Callout
      tone="warning"
      action={<Button size="xs" variant="ghost" onClick={() => onDismiss(jobs.map((job) => job.id))}>Dismiss</Button>}
    >
      {refusedText(jobs)}
    </Callout>
  );
};

export { RefusedJobs };
