/* @layer renderer-app @kind constants */
import type { IconMode } from './BoardSettings.type';

const AVATAR_SAVED = 'The lead avatar is saved.';

const AVATAR_NOTE = 'The picture this project lead wears on cards and lists. Pick a brand mark or upload an image up to 180 KB.';

const ICON_SAVED = 'The board icon is saved.';

const ICON_NOTE = 'The icon of this board in the sidebar and on its card. It is separate from the lead avatar.';

const ICON_GRID_NOTE = 'Icon grid comes with the Tessera icon picker';

const ICON_TYPES = ['image/png', 'image/svg+xml'];

const ICON_MODES: { value: IconMode; label: string }[] = [
  { value: 'brand', label: 'Brand mark' },
  { value: 'icon', label: 'Icon' },
  { value: 'upload', label: 'Upload' },
];

export { AVATAR_NOTE, AVATAR_SAVED, ICON_GRID_NOTE, ICON_MODES, ICON_NOTE, ICON_SAVED, ICON_TYPES };
