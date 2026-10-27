/* @layer renderer-app @kind component */
import { Box, Callout, Text } from '@drizztdourden08/tessera';
import { Age } from '../../../../../shell';
import { NO_BUNDLE } from '../model';
import type { LaunchBoxProps } from '../tests-props.type';
import { LaunchCommand } from './LaunchCommand';
import { LaunchNote } from './LaunchNote';

const LaunchBox = (props: LaunchBoxProps) => {
  const { view, runs, state } = props;
  const { group } = view;
  if (group.tag === NO_BUNDLE) return null;
  const draft = state === 'draft';
  const { build } = group;
  return (
    <Box className="bundle__launch" aria-label={`Launch ${group.tag}`} role="group">
      {draft && (
        <Callout tone="info">
          Draft: not built yet, so no Play and it cannot be handed to you. The lead sets coord build ready {group.tag} when it is built and testable.
        </Callout>
      )}
      <LaunchCommand view={view} runs={runs} draft={draft} />
      {build?.note && <LaunchNote tag={group.tag} note={build.note} />}
      {build?.updatedAt && <Text variant="caption" tone="muted">updated <Age value={build.updatedAt} /> ago</Text>}
    </Box>
  );
};

export { LaunchBox };
