/* @layer renderer-app @kind logic */
import type { BrandApp } from '@drizztdourden08/tessera/brand';
import { legacyKeys } from '../data/boards/legacy-keys';
import type { BoardInfo } from '../data/boards/boards.type';
import { KEY_BRANDS } from './avatar.constants';

const brandOf = (key: string, boards?: readonly BoardInfo[]): BrandApp | undefined => {
  const found = legacyKeys(key, boards).find((candidate) => Object.hasOwn(KEY_BRANDS, candidate));
  return found === undefined ? undefined : KEY_BRANDS[found];
};

export { brandOf };
