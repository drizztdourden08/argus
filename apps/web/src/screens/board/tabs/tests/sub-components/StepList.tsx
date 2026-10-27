/* @layer renderer-app @kind component */
import { Stepper } from '@drizztdourden08/tessera';
import { plainText } from '../../../plain-text';
import type { StepListProps } from '../tests-drawer-props.type';

const StepList = (props: StepListProps) => {
  const { lines } = props;
  const steps = lines.map((line, index) => ({ id: `${index}`, label: plainText(line), state: 'upcoming' as const }));
  return <Stepper steps={steps} orientation="vertical" size="sm" dotVariant="ring" label="Steps" />;
};

export { StepList };
