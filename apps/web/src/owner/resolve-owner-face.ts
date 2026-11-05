/* @layer renderer-app @kind logic */
import { AVATAR_BRANDS, BRAND_PREFIX } from '../avatar';
import { ICON_MAX } from '../board-icon';
import { INITIALS_AVATAR, OWNER_IMAGE_PATTERN } from './owner.constants';
import { DEFAULT_FACE } from './resolve-owner-face.constants';
import type { OwnerFace, OwnerProfile } from './owner.type';

const brandFace = (avatar: string): OwnerFace => {
  const app = AVATAR_BRANDS.find((brand) => brand === avatar.slice(BRAND_PREFIX.length));
  return app ? { kind: 'brand', app } : DEFAULT_FACE;
};

const resolveOwnerFace = (profile: OwnerProfile): OwnerFace => {
  const { avatar, initials } = profile;
  if (avatar === INITIALS_AVATAR) return { kind: 'initials', initials };
  if (avatar === null) return DEFAULT_FACE;
  if (avatar.startsWith(BRAND_PREFIX)) return brandFace(avatar);
  return OWNER_IMAGE_PATTERN.test(avatar) && avatar.length <= ICON_MAX ? { kind: 'image', src: avatar } : DEFAULT_FACE;
};

export { resolveOwnerFace };
