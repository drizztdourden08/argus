/* @layer renderer-app @kind component */
import { Avatar } from '@drizztdourden08/tessera';
import { BrandMark } from '@drizztdourden08/tessera/brand';
import { OWNER_ICON, resolveOwnerFace, useOwnerProfile } from '../../owner';
import type { OwnerMarkProps } from './OwnerMark.type';

const OwnerMark = (props: OwnerMarkProps) => {
  const { size = 'xs', label, profile } = props;
  const own = useOwnerProfile();
  const shown = profile ?? own;
  const title = label ?? shown.name;
  const face = resolveOwnerFace(shown);
  if (face.kind === 'brand') return <BrandMark app={face.app} size={size === 'lg' ? 'lg' : 'sm'} variant="app-icon" title={title} />;
  if (face.kind === 'image') return <Avatar src={face.src} label={title} variant="owner" size={size} />;
  if (face.kind === 'initials') return <Avatar initials={face.initials} label={title} variant="owner" size={size} />;
  return <Avatar icon={OWNER_ICON} label={title} variant="owner" size={size} />;
};

export { OwnerMark };
