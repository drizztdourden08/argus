/* @layer renderer-app @kind logic */
import type { RegistryBuild } from '../../../../../data/remote';
import { shortCommit } from '../../../../../format';
import type { BuildRow } from './model.type';

const toBuildRow = (build: RegistryBuild): BuildRow => ({
  id: `${build.name}@${build.version}@${build.commit}`,
  name: build.name,
  version: build.version,
  commit: shortCommit(build.commit),
  checkout: build.checkout,
  packedAt: build.packedAt,
  size: build.size,
});

export { toBuildRow };
