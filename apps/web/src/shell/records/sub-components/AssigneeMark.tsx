/* @layer renderer-app @kind component */
import { Avatar } from '@drizztdourden08/tessera';
import { resolveAvatar } from '../../../avatar';
import { useDoc } from '../../../data/react';
import type { BoardMeta } from '../../../data/types';
import { assigneeLook } from '../behavior/assignee-look';
import { HEAD_SIZES } from '../head.constants';
import type { AssigneeMarkProps } from '../records.type';
import { LeadMark } from './LeadMark';

const AssigneeMark = (props: AssigneeMarkProps) => {
  const { assignee, boardKey, threads, size = 'xs' } = props;
  const look = assigneeLook(assignee, boardKey, threads);
  const setting = useDoc<BoardMeta>(boardKey, 'meta', 'board').doc?.avatar;
  const variant = look.owner ? 'owner' : 'default';
  const avatar = HEAD_SIZES[size].avatar;
  const source = look.lead === undefined ? null : resolveAvatar(look.lead, look.lead === boardKey ? setting : undefined);
  if (source) return <LeadMark source={source} label={look.tip} size={size} />;
  if (look.icon) return <Avatar label={look.tip} icon={look.icon} color={look.color} variant={variant} size={avatar} />;
  if (look.initials) return <Avatar label={look.tip} initials={look.initials} color={look.color} variant={variant} size={avatar} />;
  return <Avatar label={look.tip} color={look.color} variant={variant} size={avatar} />;
};

export { AssigneeMark };
