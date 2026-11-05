/* @layer renderer-app @kind component */
import { Box, Callout, Text } from '@drizztdourden08/tessera';
import { Age } from '../../../../../shell';
import type { LaunchBoxProps } from '../tests-props.type';
import { LaunchCommand } from './LaunchCommand';
import { LaunchNote } from './LaunchNote';

const LaunchBox = (props: LaunchBoxProps) => {
  const { view, runs } = props;
  const { launch, item } = view;
  if (!view.build) return null;
  const draft = view.stage === 'draft';
  return (
    <Box className="run-card__launch" aria-label={`Launch ${item.id}`} role="group">
      {draft && (
        <Callout tone="info">
          Draft: not built yet, so no Play and it cannot be handed to you. The lead marks the run ready when its build is testable.
        </Callout>
      )}
      <LaunchCommand view={view} runs={runs} draft={draft} />
      {launch.note && <LaunchNote id={item.id} note={launch.note} />}
      {launch.updatedAt && <Text variant="caption" tone="muted">updated <Age value={launch.updatedAt} /> ago</Text>}
    </Box>
  );
};

export { LaunchBox };
