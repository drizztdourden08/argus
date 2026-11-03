/* @layer renderer-app @kind component */
import { Avatar } from '@drizztdourden08/tessera';
import { resolveAvatar } from '../../../../../avatar';
import { useDoc } from '../../../../../data/react';
import type { BoardMeta } from '../../../../../data/types';
import { HEAD_AVATAR_SIZE, HEAD_SQUARE_CLASS } from '../head.constants';
import type { AssigneeMarkProps } from '../kanban-props.type';
import { assigneeLook } from '../model';
import { LeadMark } from './LeadMark';
import './HeadRow.css';

const AssigneeMark = (props: AssigneeMarkProps) => {
  const { assignee, boardKey, threads } = props;
  const look = assigneeLook(assignee, boardKey, threads);
  const setting = useDoc<BoardMeta>(boardKey, 'meta', 'board').doc?.avatar;
  const variant = look.owner ? 'owner' : 'default';
  const className = look.lead === undefined ? undefined : HEAD_SQUARE_CLASS;
  const source = look.lead === undefined ? null : resolveAvatar(look.lead, look.lead === boardKey ? setting : undefined);
  if (source) return <LeadMark source={source} label={look.tip} />;
  if (look.icon) return <Avatar label={look.tip} icon={look.icon} color={look.color} variant={variant} size={HEAD_AVATAR_SIZE} />;
  if (look.initials) return <Avatar label={look.tip} initials={look.initials} color={look.color} variant={variant} size={HEAD_AVATAR_SIZE} className={className} />;
  return <Avatar label={look.tip} color={look.color} variant={variant} size={HEAD_AVATAR_SIZE} />;
};

export { AssigneeMark };
