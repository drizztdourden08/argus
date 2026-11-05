/* @layer renderer-app @kind logic */
import type { AvatarProps } from '@drizztdourden08/tessera';
import { assigneeLook } from '../../../../../shell';

const whoAvatar = (who: string): AvatarProps => {
  const look = assigneeLook(who, '', []);
  const base = { label: look.label, size: 'xs', color: look.color, variant: look.owner ? 'owner' : 'default' } as const;
  if (look.icon) return { ...base, icon: look.icon };
  if (look.initials) return { ...base, initials: look.initials };
  return base;
};

export { whoAvatar };
