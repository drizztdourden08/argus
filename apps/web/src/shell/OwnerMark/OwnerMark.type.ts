/* @layer renderer-app @kind types */
import type { OwnerProfile } from '../../owner';

type OwnerMarkSize = 'xs' | 'sm' | 'lg';

interface OwnerMarkProps {
  size?: OwnerMarkSize;
  label?: string;
  profile?: OwnerProfile;
}

export type { OwnerMarkProps, OwnerMarkSize };
