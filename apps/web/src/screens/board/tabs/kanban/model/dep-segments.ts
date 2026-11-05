/* @layer renderer-app @kind logic */
import type { SegmentedBarSegment } from '@drizztdourden08/tessera';
import type { ItemRef } from '../../../../../data/kanban';
import { depLinks } from './dep-links';
import { depTypeOf } from './dep-type-of';
import { SETTLED_DEPS } from './model.constants';
import { typeTone } from './type-tone';

const depSegments = (item: ItemRef): SegmentedBarSegment[] =>
  depLinks(item)
    .filter((link) => !SETTLED_DEPS.includes(link.state))
    .map((link) => ({ tone: typeTone(depTypeOf(link.item), link.state === 'done'), label: `${link.item} ${link.title}, ${link.state}` }));

export { depSegments };
