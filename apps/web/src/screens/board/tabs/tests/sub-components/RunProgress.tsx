/* @layer renderer-app @kind component */
import { StackedBar } from '@drizztdourden08/tessera';
import { shownSegments } from '../behavior/shown-segments';
import { countsText } from '../model';
import type { RunProgressProps } from '../tests-props.type';

const RunProgress = (props: RunProgressProps) => {
  const { counts, fill = false } = props;
  const segments = shownSegments(counts);
  if (segments.length === 0) return null;
  return <StackedBar className={fill ? undefined : 'run-card__bar'} segments={segments} size="sm" legend={false} label={countsText(counts)} />;
};

export { RunProgress };
