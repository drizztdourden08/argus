/* @layer renderer-app @kind types */
import type { BrandApp } from '@drizztdourden08/tessera/brand';
import type { BoardIconName } from '../board-icon/board-icon.type';

type AvatarSource = { kind: 'brand'; app: BrandApp } | { kind: 'icon'; name: BoardIconName } | { kind: 'image'; src: string };

export type { AvatarSource };
