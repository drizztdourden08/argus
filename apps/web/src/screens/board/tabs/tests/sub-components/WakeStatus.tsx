/* @layer renderer-app @kind component */
import { Tag } from '@drizztdourden08/tessera';
import { useTestsShared } from '../behavior/useTestsShared';
import { wakeChip } from '../model';
import { WAKE_TAGS } from '../tests.constants';
import type { TestPartProps } from '../tests-props.type';

const WakeStatus = (props: TestPartProps) => {
  const { wakes } = useTestsShared();
  const chip = wakeChip({ test: props.test, wakes: wakes.all, latest: wakes.latest, now: wakes.now });
  if (!chip) return null;
  return <Tag {...WAKE_TAGS[chip.tone]} title={chip.title} data-pulse={chip.pulse ? '' : undefined}>{chip.label}</Tag>;
};

export { WakeStatus };
