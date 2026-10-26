/* @layer renderer-app @kind component */
import { Status } from '@drizztdourden08/tessera';
import { useTestsShared } from '../behavior/useTestsShared';
import { wakeChip } from '../model';
import type { TestPartProps } from '../tests-props.type';

const WakeStatus = (props: TestPartProps) => {
  const { wakes } = useTestsShared();
  const chip = wakeChip({ test: props.test, wakes: wakes.all, latest: wakes.latest, now: wakes.now });
  if (!chip) return null;
  return <Status tone={chip.tone} dot pulse={chip.pulse} title={chip.title}>{chip.label}</Status>;
};

export { WakeStatus };
