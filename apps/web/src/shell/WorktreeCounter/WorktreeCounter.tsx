/* @layer renderer-app @kind component */
import { Tag } from '@drizztdourden08/tessera';
import { SLACK } from './WorktreeCounter.constants';
import type { WorktreeCounterProps } from './WorktreeCounter.type';

const WorktreeCounter = (props: WorktreeCounterProps) => {
  const { listed, running } = props;
  const over = listed > running + SLACK;
  const title = over
    ? `${listed} worktrees are listed for ${running} running agents: some are not cleaned. Open the Worktrees tab.`
    : 'Worktrees listed (main checkouts aside) against the agents running now. About one to one is healthy.';
  const text = `worktrees ${listed} / running agents ${running}`;
  if (over) return <Tag variant="urgency" color="warning" title={title}>{text}</Tag>;
  return <Tag title={title}>{text}</Tag>;
};

export { WorktreeCounter };
