/* @layer renderer-app @kind component */
import { Avatar } from '@drizztdourden08/tessera';
import { OwnerMark } from '../../OwnerMark';
import { resolveAvatar } from '../../../avatar';
import { useDoc } from '../../../data/react';
import type { BoardMeta } from '../../../data/types';
import { assigneeLook } from '../behavior/assignee-look';
import type { AssigneeMarkProps } from '../records.type';
import { LeadMark } from './LeadMark';

const AssigneeMark = (props: AssigneeMarkProps) => {
  const { assignee, boardKey, threads, size = 'xs' } = props;
  const look = assigneeLook(assignee, boardKey, threads);
  const setting = useDoc<BoardMeta>(boardKey, 'meta', 'board').doc?.avatar;
  if (look.owner) return <OwnerMark label={look.label} size={size} />;
  const source = look.lead === undefined ? null : resolveAvatar(look.lead, look.lead === boardKey ? setting : undefined);
  if (source) return <LeadMark source={source} label={look.tip} size={size} />;
  if (look.icon) return <Avatar label={look.tip} icon={look.icon} color={look.color} variant="default" size={size} />;
  if (look.initials) return <Avatar label={look.tip} initials={look.initials} color={look.color} variant="default" size={size} />;
  return <Avatar label={look.tip} color={look.color} variant="default" size={size} />;
};

export { AssigneeMark };
