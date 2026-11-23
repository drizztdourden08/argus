/* @layer renderer-app @kind component */
import { Box } from '@drizztdourden08/tessera';
import { stageViews } from '../model';
import type { ProcessDocProps } from '../switch.type';
import { StageRow } from './StageRow';

const StageTimeline = (props: ProcessDocProps) => (
  <Box as="ol" className="stage-timeline" aria-label="Switch stages">
    {stageViews(props.doc).map((view) => <StageRow key={view.stage} view={view} doc={props.doc} />)}
  </Box>
);

export { StageTimeline };
