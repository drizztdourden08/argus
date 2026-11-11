/* @layer renderer-app @kind component */
import { Inline } from '@drizztdourden08/tessera';
import type { StepKindTagsProps } from '../compare.type';
import { stepKinds } from '../model';
import { StatusChip } from './StatusChip';

const StepKindTags = (props: StepKindTagsProps) => (
  <Inline gap="sm" align="center" wrap>
    {stepKinds(props.steps).map((look) => <StatusChip key={look.label} look={look} />)}
  </Inline>
);

export { StepKindTags };
