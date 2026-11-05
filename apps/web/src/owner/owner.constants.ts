/* @layer renderer-app @kind constants */
import type { OwnerProfile } from './owner.type';

const OWNER_DEFAULTS: OwnerProfile = { name: 'You', initials: 'ME', avatar: null };

const OWNER_ICON = 'user';

const INITIALS_AVATAR = 'initials';

const OWNER_IMAGE_PATTERN = /^(data:image\/(png|svg\+xml);base64,|\/api\/owner\/avatar\?v=[0-9a-f]*$)/;

export { INITIALS_AVATAR, OWNER_DEFAULTS, OWNER_ICON, OWNER_IMAGE_PATTERN };
