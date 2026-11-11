/* @layer renderer-app @kind constants */
import type { MenuItem } from '@drizztdourden08/tessera';
import type { Side } from '../review.type';

const SIDE_ITEMS: readonly { side: Side; id: string; label: string; icon: MenuItem['icon'] }[] = [
  { side: 'left', id: 'take-left', label: 'Take left for all', icon: 'arrow-left' },
  { side: 'right', id: 'take-right', label: 'Take right for all', icon: 'arrow-right' },
  { side: 'skip', id: 'skip-all', label: 'Skip all', icon: 'pause' },
];

export { SIDE_ITEMS };
