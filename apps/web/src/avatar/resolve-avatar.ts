/* @layer renderer-app @kind logic */
import { AVATAR_BRANDS, AVATAR_MAX_CHARS, BRAND_PREFIX, IMAGE_PREFIX } from './avatar.constants';
import { brandOf } from './brand-of';
import type { BoardInfo } from '../data/boards/boards.type';
import type { AvatarSource } from './avatar.type';

const fromSetting = (setting: string): AvatarSource | null => {
  if (setting.startsWith(BRAND_PREFIX)) {
    const app = AVATAR_BRANDS.find((brand) => brand === setting.slice(BRAND_PREFIX.length));
    return app ? { kind: 'brand', app } : null;
  }
  return setting.startsWith(IMAGE_PREFIX) && setting.length <= AVATAR_MAX_CHARS ? { kind: 'image', src: setting } : null;
};

const resolveAvatar = (key: string, setting?: unknown, boards?: readonly BoardInfo[]): AvatarSource | null => {
  const chosen = typeof setting === 'string' ? fromSetting(setting) : null;
  if (chosen) return chosen;
  const app = brandOf(key, boards);
  return app ? { kind: 'brand', app } : null;
};

export { resolveAvatar };
