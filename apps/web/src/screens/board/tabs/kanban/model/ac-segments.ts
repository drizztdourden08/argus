/* @layer renderer-app @kind logic */
import type { SegmentedBarSegment } from '@drizztdourden08/tessera';
import type { ItemRef } from '../../../../../data/kanban';

const acSegments = (criteria: NonNullable<ItemRef['ac']>): SegmentedBarSegment[] =>
  criteria.map((criterion, index) => ({
    tone: criterion.done ? 'success' : 'track',
    label: `${index + 1}. ${criterion.text}, ${criterion.done ? 'met' : 'open'}`,
  }));

export { acSegments };
