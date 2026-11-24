/* @layer renderer-app @kind component */
import { Stepper, Text } from '@drizztdourden08/tessera';
import { nextSteps } from '../model';
import { RUN_IDLE, STRIP_LABEL } from '../next.constants';
import type { NextRunProps } from '../next.type';

const NextRun = (props: NextRunProps) => {
  const view = nextSteps(props.phase);
  if (!view.currentId && !view.complete) return <Text variant="label">{RUN_IDLE}</Text>;
  return <Stepper {...view} compact size="sm" label={STRIP_LABEL} />;
};

export { NextRun };
