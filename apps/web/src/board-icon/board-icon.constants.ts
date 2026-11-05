/* @layer renderer-app @kind constants */
const BOARD_ICON_NAMES = [
  'rocket',
  'boxes',
  'hammer',
  'flask-conical',
  'gamepad-2',
  'palette',
  'brain',
  'globe',
  'terminal',
  'database',
  'shield',
  'zap',
  'leaf',
  'music',
  'camera',
  'book-open',
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
