/* @layer renderer-app @kind logic */
import { AVATAR_BRANDS, BRAND_PREFIX, brandOf } from '../avatar';
import { firstLegacyKey } from '../data/boards';
import { BOARD_ICON_NAMES, DEFAULT_ICON_COUNT, ICON_MAX, ICON_NAME_PREFIX, ICON_URL_PATTERN, PNG_PREFIX, SVG_PREFIX } from './board-icon.constants';
import { toBoardIconName } from './to-board-icon-name';
import type { BoardIconSource, ResolvedBoardIcon } from './board-icon.type';

const parseBrand = (value: string): BoardIconSource | null => {
  const app = AVATAR_BRANDS.find((brand) => brand === value.slice(BRAND_PREFIX.length));
  return app ? { kind: 'brand', app } : null;
};

const parseName = (value: string): BoardIconSource | null => {
  const name = toBoardIconName(value.slice(ICON_NAME_PREFIX.length));
  return name ? { kind: 'icon', name } : null;
};

const isImage = (value: string): boolean =>
  (value.startsWith(PNG_PREFIX) || value.startsWith(SVG_PREFIX)) && value.length <= ICON_MAX;

const parseSetting = (value: unknown): BoardIconSource | null => {
  if (typeof value !== 'string') return null;
  if (value.startsWith(BRAND_PREFIX)) return parseBrand(value);
  if (value.startsWith(ICON_NAME_PREFIX)) return parseName(value);
  return isImage(value) || ICON_URL_PATTERN.test(value) ? { kind: 'image', src: value } : null;
};

const hashKey = (key: string): number => [...key].reduce((hash, char) => (hash * 31 + char.charCodeAt(0)) >>> 0, 7);

const defaultSource = (key: string, avatar: unknown): BoardIconSource => {
  const app = brandOf(key);
  if (app) return { kind: 'brand', app };
  const brandAvatar = typeof avatar === 'string' && avatar.startsWith(BRAND_PREFIX) ? parseBrand(avatar) : null;
  return brandAvatar ?? { kind: 'icon', name: BOARD_ICON_NAMES[hashKey(firstLegacyKey(key)) % DEFAULT_ICON_COUNT] ?? 'code' };
};

const resolveBoardIcon = (key: string, icon?: unknown, avatar?: unknown, cardIcon?: unknown): ResolvedBoardIcon => {
  const chosen = parseSetting(icon) ?? parseSetting(cardIcon);
  return chosen ? { source: chosen, isDefault: false } : { source: defaultSource(key, avatar), isDefault: true };
};

export { resolveBoardIcon };
