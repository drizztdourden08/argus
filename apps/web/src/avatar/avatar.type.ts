/* @layer renderer-app @kind types */
import type { BrandApp } from '@drizztdourden08/tessera/brand';

type AvatarSource = { kind: 'brand'; app: BrandApp } | { kind: 'image'; src: string };

export type { AvatarSource };
