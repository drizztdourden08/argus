/* @layer renderer-app @kind logic */
import type { BuildDoc, BundleStatus } from './model.type';

const bundleStatus = (build: BuildDoc | null): BundleStatus => {
  if (build?.closedAt) return 'closed';
  return build?.status === 'draft' ? 'draft' : 'ready';
};

export { bundleStatus };
