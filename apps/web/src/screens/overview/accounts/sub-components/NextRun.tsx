/* @layer renderer-app @kind component */
import { Stepper } from '@drizztdourden08/tessera';
import { nextSteps } from '../model';
import { STRIP_LABEL } from '../next.constants';
import type { NextRunProps } from '../next.type';

const NextRun = (props: NextRunProps) => <Stepper {...nextSteps(props.phase)} compact size="sm" label={STRIP_LABEL} />;

export { NextRun };
