/* @layer renderer-app @kind util */
import { columnWidth } from './column-width';

const agentWidths = {
  status: columnWidth(6.8, 40, 1.4),
  agent: columnWidth(9.5, 0),
  model: columnWidth(4.4, 0),
  kind: columnWidth(4, 70),
  item: columnWidth(3.6, 56),
  branch: columnWidth(7, 80),
  time: columnWidth(4.2, 0),
  now: columnWidth(10, 70),
  actions: columnWidth(8, 0),
} as const;

export { agentWidths };
