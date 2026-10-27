/* @layer renderer-app @kind logic */
import type { Build, BundleState } from '../../../../data/types';

const bundleState = (build: Build | null | undefined): BundleState => {
  if (build?.closedAt) return 'closed';
  return build?.status === 'draft' ? 'draft' : 'ready';
};

export { bundleState };
