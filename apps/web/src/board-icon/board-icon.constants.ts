/* @layer renderer-app @kind constants */
const BOARD_ICON_NAMES = [
  'gamepad-2',
  'palette',
  'flask',
  'book',
  'globe',
  'database',
  'code',
  'cpu',
  'package',
  'compass',
  'layers',
  'network',
  'music',
  'camera',
  'heart',
  'zap',
  'star',
  'flag',
  'folder',
  'wrench',
  'puzzle',
  'sparkles',
  'server',
  'lock',
  'bell',
  'calendar',
  'bug',
  'trophy',
  'sun',
  'moon',
  'home',
  'message-square',
  'users',
  'bookmark',
  'headphones',
  'image',
  'mic',
  'clock',
  'mail',
  'map',
  'monitor',
  'smartphone',
  'gift',
  'tag',
  'link',
] as const;

const ICON_MAX = 64 * 1024;

const SVG_MAX = 32 * 1024;

const PNG_PICK_MAX = 1024 * 1024;

const ICON_PX = 128;

const PNG_PREFIX = 'data:image/png;base64,';

const SVG_PREFIX = 'data:image/svg+xml;base64,';

const ICON_URL_PATTERN = /^\/api\/p\/[^/?#]+\/icon\?v=[0-9a-f]*$/;

const ICON_NAME_PREFIX = 'icon:';

export {
  BOARD_ICON_NAMES,
  ICON_MAX,
  ICON_NAME_PREFIX,
  ICON_PX,
  ICON_URL_PATTERN,
  PNG_PICK_MAX,
  PNG_PREFIX,
  SVG_MAX,
  SVG_PREFIX,
};
