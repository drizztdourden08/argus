/* @layer renderer-app @kind component */
import { Icon, IconButton } from '@drizztdourden08/tessera';
import { copyText } from '../../../shared';
import type { AgentBranchButtonProps } from '../row-props.type';

const AgentBranchButton = (props: AgentBranchButtonProps) => {
  const { branch } = props;
  if (!branch) return null;
  return (
    <IconButton label={`Copy the branch: ${branch}`} variant="ghost" tone="info" size="sm" className="agents-actions__extra" onClick={() => copyText(branch)}>
      <Icon name="git-fork" />
    </IconButton>
  );
};

export { AgentBranchButton };
