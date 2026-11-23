/* @layer renderer-app @kind constants */
import type { IconMode } from './BoardSettings.type';

const AVATAR_SAVED = 'The lead avatar is saved.';

const AVATAR_NOTE = 'The picture this project lead wears on cards and lists. Pick a brand mark or upload an image up to 180 KB.';

const ICON_SAVED = 'The board icon is saved.';

const ICON_NOTE = 'The icon of this board in the sidebar and on its card. It is separate from the lead avatar.';

const ICON_VALUE_PREFIX = 'icon:';

const ICON_PICKER_LABEL = 'Pick a board icon';

const ICON_TYPES = ['image/png', 'image/svg+xml'];

const ICON_MODES: { value: IconMode; label: string }[] = [
  { value: 'brand', label: 'Brand mark' },
  { value: 'icon', label: 'Icon' },
  { value: 'upload', label: 'Upload' },
];

const TITLE_LIMIT_SAVED = 'The title length limit is saved.';

const TITLE_LIMIT_NOTE = 'Longest item title this board accepts, in characters.';

const TITLE_LIMIT_DEFAULT = 80;

const TITLE_LIMIT_MIN = 20;

const TITLE_LIMIT_MAX = 200;

const ICON_REFUSED = 'Only PNG and SVG';
const AVATAR_REFUSED = 'Only PNG, JPEG, WebP and GIF';

export { AVATAR_NOTE, AVATAR_REFUSED, AVATAR_SAVED, ICON_REFUSED, ICON_MODES, ICON_NOTE, ICON_PICKER_LABEL, ICON_SAVED, ICON_VALUE_PREFIX, ICON_TYPES, TITLE_LIMIT_DEFAULT, TITLE_LIMIT_MAX, TITLE_LIMIT_MIN, TITLE_LIMIT_NOTE, TITLE_LIMIT_SAVED };
