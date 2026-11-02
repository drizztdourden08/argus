/* @layer renderer-app @kind logic */
import type { DepColor, DepState } from './deps.type';

const COLORS: Readonly<Record<DepState, DepColor>> = {
  opening: 'info',
  open: 'info',
  blocked: 'warning',
  done: 'success',
  dropped: 'neutral',
  gone: 'neutral',
  cancelled: 'neutral',
};

const depColor = (state: DepState): DepColor => COLORS[state] ?? 'neutral';

export { depColor };
