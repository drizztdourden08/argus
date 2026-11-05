/* @layer renderer-app @kind logic */
import type { Counts } from '../model';
import { PROGRESS_SEGMENTS } from '../tests.constants';

const shownSegments = (counts: Counts) =>
  PROGRESS_SEGMENTS.filter((segment) => counts[segment.id] > 0).map((segment) => ({ ...segment, value: counts[segment.id] }));

export { shownSegments };
