/* @layer renderer-app @kind logic */
import type { SegmentedBarSegment } from '@drizztdourden08/tessera';
import { itemCode } from '../../../../../shell';
import type { ChildCount } from './model.type';
import { typeOf } from './type-of';
import { typeTone } from './type-tone';

const childSegments = (children: ChildCount): SegmentedBarSegment[] =>
  children.items.map((child) => ({
    tone: typeTone(typeOf(child), child.status === 'done'),
    label: `${itemCode(child)} ${child.title}, ${child.status}`,
  }));

export { childSegments };
