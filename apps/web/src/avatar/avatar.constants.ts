/* @layer renderer-app @kind constants */
import type { BrandApp } from '@drizztdourden08/tessera/brand';

const AVATAR_BRANDS: readonly BrandApp[] = ['argus', 'tessera', 'rotp', 'archipelia', 'brock', 'bxel'];

const KEY_BRANDS: Readonly<Record<string, BrandApp>> = {
  argus: 'argus',
  tessera: 'tessera',
  rotp: 'rotp',
  archipelia: 'archipelia',
  brock: 'brock',
  pixelarttool: 'bxel',
};

const BRAND_PREFIX = 'brand:';

const IMAGE_PREFIX = 'data:image/';

const AVATAR_MAX_CHARS = 240 * 1024;

const AVATAR_TYPES = ['image/png', 'image/jpeg', 'image/webp', 'image/gif'];

export { AVATAR_BRANDS, AVATAR_MAX_CHARS, AVATAR_TYPES, BRAND_PREFIX, IMAGE_PREFIX, KEY_BRANDS };
