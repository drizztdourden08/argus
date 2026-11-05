/* @layer renderer-app @kind constants */
import type { OwnerMarkSize } from '../OwnerMark/OwnerMark.type';
import type { AvatarMode, SectionId } from './AppSettings.type';

const SECTIONS: { value: SectionId; label: string }[] = [
  { value: 'profile', label: 'Profile' },
  { value: 'notifications', label: 'Notifications' },
  { value: 'appearance', label: 'Appearance' },
  { value: 'system', label: 'System' },
];

const FIRST_SECTION: SectionId = 'profile';

const AVATAR_MODES: { value: AvatarMode; label: string }[] = [
  { value: 'default', label: 'Default' },
  { value: 'initials', label: 'Initials' },
  { value: 'brand', label: 'Brand mark' },
  { value: 'upload', label: 'Upload' },
];

const MODE_OF_KIND: Record<string, AvatarMode> = { default: 'default', initials: 'initials', brand: 'brand', image: 'upload' };

const PREVIEW_SIZE: OwnerMarkSize = 'lg';

const INITIALS_LENGTH = 2;

const PROFILE_SAVED = 'Your profile is saved.';

const AVATAR_NOTE = 'The picture you wear on cards, lists and the shell bar. Pick a brand mark or upload a PNG or SVG.';

const NAME_NOTE = 'Shown in tooltips and in the assignee picker. Leave it empty to go back to You.';

const INITIALS_NOTE = 'Two characters for the Initials avatar. Leave it empty to go back to ME.';

const UPLOAD_TYPES = ['image/png', 'image/svg+xml'];
const UPLOAD_REFUSED = 'Only PNG and SVG';

export {
  AVATAR_MODES,
  AVATAR_NOTE,
  FIRST_SECTION,
  INITIALS_LENGTH,
  INITIALS_NOTE,
  MODE_OF_KIND,
  NAME_NOTE,
  PREVIEW_SIZE,
  PROFILE_SAVED,
  SECTIONS,
  UPLOAD_REFUSED,
  UPLOAD_TYPES,
};
