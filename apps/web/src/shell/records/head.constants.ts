/* @layer renderer-app @kind constants */
import type { HeadSize } from './records.type';

const HEAD_SIZE_PX = 16;

const HEAD_AVATAR_SIZE = 'xs';

const HEAD_SIZES: Readonly<Record<HeadSize, { px: number; avatar: 'xs' | 'sm' }>> = {
  xs: { px: HEAD_SIZE_PX, avatar: HEAD_AVATAR_SIZE },
  sm: { px: 20, avatar: 'sm' },
};

const MENU_ICON = 'ellipsis-vertical';

export { HEAD_AVATAR_SIZE, HEAD_SIZE_PX, HEAD_SIZES, MENU_ICON };
