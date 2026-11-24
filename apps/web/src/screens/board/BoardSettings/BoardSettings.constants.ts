/* @layer renderer-app @kind constants */
import type { SettingsTab } from './BoardSettings.type';

const AVATAR_SAVED = 'The lead avatar is saved.';

const AVATAR_NOTE = 'The picture this project lead wears on cards and lists. Pick an icon or upload an image up to 180 KB.';

const ICON_SAVED = 'The board icon is saved.';

const ICON_NOTE = 'The icon of this board in the sidebar and on its card. Pick an icon or upload a PNG or SVG. It is separate from the lead avatar.';

const ICON_VALUE_PREFIX = 'icon:';

const ICON_PICKER_LABEL = 'Pick a board icon';

const AVATAR_PICKER_LABEL = 'Pick a lead avatar icon';

const ICON_TYPES = ['image/png', 'image/svg+xml'];

const SETTINGS_TABS: { id: SettingsTab; label: string }[] = [
  { id: 'appearance', label: 'Appearance' },
  { id: 'general', label: 'General' },
  { id: 'kanban', label: 'Kanban' },
];

const TITLE_LIMIT_SAVED = 'The title length limit is saved.';

const TITLE_LIMIT_NOTE = 'Longest item title this board accepts, in characters.';

const TITLE_LIMIT_DEFAULT = 80;

const TITLE_LIMIT_MIN = 20;

const TITLE_LIMIT_MAX = 200;

const ICON_REFUSED = 'Only PNG and SVG';
const AVATAR_REFUSED = 'Only PNG, JPEG, WebP and GIF';

export { AVATAR_NOTE, AVATAR_PICKER_LABEL, AVATAR_REFUSED, AVATAR_SAVED, ICON_REFUSED, ICON_NOTE, ICON_PICKER_LABEL, ICON_SAVED, ICON_VALUE_PREFIX, ICON_TYPES, SETTINGS_TABS, TITLE_LIMIT_DEFAULT, TITLE_LIMIT_MAX, TITLE_LIMIT_MIN, TITLE_LIMIT_NOTE, TITLE_LIMIT_SAVED };
