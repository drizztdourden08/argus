/* @layer renderer-app @kind component */
import { StackedBar } from '@drizztdourden08/tessera';
import type { StackedBarSegment } from '@drizztdourden08/tessera';
import { countsText } from '../model';
import { PROGRESS_SEGMENTS } from '../tests.constants';
import type { RunProgressProps } from '../tests-props.type';

const RunProgress = (props: RunProgressProps) => {
  const { counts, fill = false } = props;
  const segments: StackedBarSegment[] = PROGRESS_SEGMENTS
    .filter((segment) => counts[segment.id] > 0)
    .map((segment) => ({ id: segment.id, label: segment.label, value: counts[segment.id], color: segment.color }));
  if (segments.length === 0) return null;
  return <StackedBar className={fill ? undefined : 'run-card__bar'} segments={segments} size="sm" legend={false} label={countsText(counts)} />;
};

export { RunProgress };
