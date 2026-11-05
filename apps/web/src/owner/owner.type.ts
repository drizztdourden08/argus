/* @layer renderer-app @kind types */
import type { BrandApp } from '@drizztdourden08/tessera/brand';

interface OwnerProfile {
  name: string;
  initials: string;
  avatar: string | null;
}

type OwnerFace =
  | { kind: 'default' }
  | { kind: 'initials'; initials: string }
  | { kind: 'brand'; app: BrandApp }
  | { kind: 'image'; src: string };

export type { OwnerFace, OwnerProfile };
