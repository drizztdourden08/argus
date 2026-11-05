/* @layer renderer-app @kind logic */
import type { SignalItem } from '@drizztdourden08/tessera';
import { OWNER_ICON, resolveOwnerFace } from '../../../../../owner';
import type { OwnerProfile } from '../../../../../owner';

const ownerMark = (profile: OwnerProfile): NonNullable<SignalItem['mark2']> => {
  const face = resolveOwnerFace(profile);
  const base = { label: profile.name, size: 'xs', variant: 'owner' } as const;
  if (face.kind === 'brand') return { brand: face.app };
  if (face.kind === 'image') return { avatar: { ...base, src: face.src } };
  if (face.kind === 'initials') return { avatar: { ...base, initials: face.initials } };
  return { avatar: { ...base, icon: OWNER_ICON } };
};

export { ownerMark };
