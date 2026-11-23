/* @layer renderer-app @kind logic */
import type { TagLook } from '@drizztdourden08/tessera';
import type { StatusTone } from '@drizztdourden08/tessera';

const tagLook = (tone: StatusTone): TagLook => {
  if (tone === 'success' || tone === 'warning' || tone === 'danger' || tone === 'info') return { variant: 'urgency', color: tone };
  return { variant: 'normal', color: 'neutral' };
};

export { tagLook };
