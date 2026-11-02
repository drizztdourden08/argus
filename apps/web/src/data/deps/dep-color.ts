/* @layer renderer-app @kind logic */
import { DEP_COLORS } from './deps.constants';
import type { DepColor, DepState } from './deps.type';

const depColor = (state: DepState): DepColor => DEP_COLORS[state];

export { depColor };
