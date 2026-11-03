/* @layer renderer-app @kind logic */
import type { CardProgress, ChildCount } from './model.type';

const childrenProgress = (children: ChildCount): CardProgress | null =>
  children.total === 0
    ? null
    : { met: children.done, total: children.total, text: `${children.done}/${children.total}`, title: `${children.done} of ${children.total} children done` };

export { childrenProgress };
