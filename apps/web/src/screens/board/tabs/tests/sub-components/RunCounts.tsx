/* @layer renderer-app @kind component */
import { SignalStrip } from '@drizztdourden08/tessera';
import type { SignalItem } from '@drizztdourden08/tessera';
import { shownSegments } from '../behavior/shown-segments';
import { countsText } from '../model';
import type { RunProgressProps } from '../tests-props.type';

const RunCounts = (props: RunProgressProps) => {
  const { counts } = props;
  const items: SignalItem[] = shownSegments(counts).map((segment) => ({
    id: segment.id,
    icon: segment.icon,
    count: segment.value,
    label: `${segment.value} ${segment.label}`,
    tone: segment.color,
  }));
  return <SignalStrip items={items} wrap aria-label={countsText(counts)} className="run-card__counts" />;
};

export { RunCounts };
