/* @layer renderer-app @kind component */
import { Avatar } from '@drizztdourden08/tessera';
import { assigneeLook } from '../../model';
import type { TreeRowProps } from '../tree.type';

const TreeAssignee = (props: TreeRowProps) => {
  const { node, boardKey, threads } = props;
  const look = assigneeLook(node.assignee, boardKey, threads);
  const variant = look.owner ? 'owner' : 'default';
  if (look.icon) return <Avatar label={look.tip} icon={look.icon} color={look.color} variant={variant} size="xs" />;
  if (look.initials) return <Avatar label={look.tip} initials={look.initials} color={look.color} variant={variant} size="xs" />;
  return <Avatar label={look.tip} color={look.color} variant={variant} size="xs" />;
};

export { TreeAssignee };
