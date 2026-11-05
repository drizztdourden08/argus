/* @layer renderer-app @kind logic */
import type { SegmentedBarTone } from '@drizztdourden08/tessera';
import { itemTypeOf } from '../../../../../data/registry';
import { categoryColor } from '../../../../../shell';

const typeTone = (typeId: string, done: boolean): SegmentedBarTone => (done ? (categoryColor(itemTypeOf(typeId)?.color) ?? 'success') : 'track');

export { typeTone };
