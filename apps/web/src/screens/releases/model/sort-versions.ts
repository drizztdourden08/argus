/* @layer renderer-app @kind logic */
import type { ReleaseVersion } from '../../../data/remote';
import { toMillis } from '../../../format';

const stamp = (version: ReleaseVersion): number => {
  const at = toMillis(version.publishedAt);
  return Number.isFinite(at) ? at : 0;
};

const sortVersions = (versions: readonly ReleaseVersion[]): ReleaseVersion[] => [...versions].sort((left, right) => stamp(right) - stamp(left));

export { sortVersions };
